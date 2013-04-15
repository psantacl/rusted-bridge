extern mod std;
extern mod wol;

use wol::property_file::read_file;
use std::uv_iotask;
use std::net_tcp;
use std::net_ip::v4;
use std::uv;
use str::raw::from_c_str;
use str::StrSlice;

use core::result::{Ok,Err};
use std::getopts::{optopt, getopts, opt_maybe_str, fail_str };
use std::json::{ToJson,from_str,Error,Json, Object,to_str, Decoder};
use libc::{c_char};
use task::spawn;
use pipes::{stream, Port, Chan};
use io::{WriterUtil,ReaderUtil};
use core::send_map::linear::{LinearMap};

mod daemon_tools;

fn poll_for_connection(host: &str, port: &str) -> (std::net_tcp::TcpSocket) {
  let mut count = 0;

  while count < 10 {
    let io_task = uv::global_loop::get();
    let conn_res : Result<std::net_tcp::TcpSocket,std::net_tcp::TcpConnectErrData> = std::net_tcp::connect(std::net_ip::v4::parse_addr(host), 
        option::unwrap(uint::from_str(port)), 
        io_task);
    if (!conn_res.is_err()) {
      return  core::result::unwrap(conn_res);
    }
    libc::funcs::posix88::unistd::sleep(1);
    count += 1;
  }
  fail(fmt!("failed to connect to service after %d attempts", count));
}

fn ensure_connection(host: ~str, port: ~str, bridge : &daemon_tools::Bridge) -> (std::net_tcp::TcpSocket) {
  let io_task = uv::global_loop::get();
  let conn_res : Result<std::net_tcp::TcpSocket,std::net_tcp::TcpConnectErrData> = std::net_tcp::connect( std::net_ip::v4::parse_addr(host.to_managed()), 
      option::unwrap(uint::from_str(port.to_managed())), 
      io_task);
  if conn_res.is_err() {
    let pid = libc::funcs::posix88::unistd::fork();
    if (pid < 0) {
      fail ~"Error: unable to fork whilst trying to launch jvm"
    } else if (pid == 0) {
      daemon_tools::daemonize(bridge);
    } else {
      return poll_for_connection(host,port);
    }
  } 
  core::result::unwrap(conn_res)
}


#[allow(non_implicitly_copyable_typarams)]
fn parse_cmd_arguments() -> (path::Path, ~str,~str) {
  let args = os::args();
  let homedir = match os::homedir() {
    None       => { fail(~"could not determine users home dir to find config file"); }
    Some(path) =>  { path }
  };

  let inferred_input_file = homedir.push(".rusted-bridge").push( args[1] ).push("config");
  let pid_file = homedir.push(".rusted-bridge").push( args[1] ).push("pid");

  do str::as_c_str(inferred_input_file.to_str()) |input_file| {
    if ( libc::funcs::posix88::unistd::access(input_file,  libc::consts::os::posix88::R_OK as core::libc::types::os::arch::c95::c_int) != 0) {
      fail(fmt!("unable to read config file %s", inferred_input_file.to_str()));
    }
  }

  let bridge_cmd = str::connect(vec::slice(args,2, vec::len(args)), &" ");

  io::println(fmt!("bridge_cmd %s", bridge_cmd));
  return (pid_file, inferred_input_file.to_str(), bridge_cmd);
}



#[allow(non_implicitly_copyable_typarams)]
fn parse_cmd(next_cmd : &str) -> (Option<(~Object,~str)>) {
  let mut idx = 1;
  while (idx < next_cmd.len() + 1) {
    let framed_cmd = core::str::slice(next_cmd,0,idx);
    //io::println(fmt!("framed cmd: %s", framed_cmd));
    let parse_result : Result<Json,Error> = std::json::from_str(framed_cmd);
    if (!parse_result.is_err()) {
      let json_response = match( core::result::unwrap(parse_result) ) {
        Object(cmd) => { cmd }
        _           => { fail(~"received command was wrong type"); }
      };
      let remaining_cmd = core::str::slice(next_cmd,idx,next_cmd.len());
      return Some((json_response, remaining_cmd));
    } else {
      idx += 1;
    }
  }
  return None;
}

#[allow(non_implicitly_copyable_typarams)]
fn event_loop(da_socket : std::net_tcp::TcpSocket, 
              std_out_channel : Chan<Option<~str>>,
              std_err_channel : Chan<Option<~str>>,
              std_in_port     : Port<Option<~str>>) -> () {

  let mut next_cmd : ~str = ~"";
  let result = std::net_tcp::read_start( &da_socket );

  let socket_port = match(result.is_err()) {
    false => { core::result::unwrap(result) }
    true  => { fail(~"Error reading socket"); }
  };

  loop {
    if (socket_port.peek()) {
      let next_result = socket_port.recv();
      let next_read = match (next_result.is_err())  {
        true => { 
         let err_data = result::unwrap_err(next_result);
         if err_data.err_name == ~"EOF" {
            libc::funcs::c95::stdlib::exit(0);
           std_out_channel.send(None);
           std_err_channel.send(None);
           break;
         } else {
           fail ~"Error reading socket" 
         } 
        }
        false => { core::result::unwrap(next_result) } 
      };

      next_cmd = core::str::append(next_cmd, core::str::from_bytes(next_read));
      loop {
        let (json_response,remaining_cmd) = match(parse_cmd(next_cmd)) {
          None => { break; } 
          Some((json_response,remaining_cmd)) => { (json_response,remaining_cmd) }
        };
        next_cmd = remaining_cmd;
        let cmd_str = match (json_response.find(&~"command")) {
          None      => { fail(~"response malformed. no command found") }
          Some(cmd) =>  { Decoder(cmd).read_owned_str() } 
        };

        let payload_str = match (json_response.find(&~"payload")) {
          None => { fail(~"response malformed. no payload found") }
          Some(payload) =>  { Decoder(payload).read_owned_str() } 
        };

        match (cmd_str) {
          ~"std-out" => { std_out_channel.send(Some(payload_str)); }
          ~"std-err" => { std_err_channel.send(Some(payload_str)); }
          _          => { fail(fmt!("unrecognized command %s", cmd_str)); }
        }
      }     
    } 

    if (std_in_port.peek()) {
      let cmd = std_in_port.recv();
      match cmd {
        None => { break; }
        Some(payload) => { 
          let mut cmd_map : LinearMap<~str,~str> = LinearMap();
          if (!cmd_map.insert( ~"cmd", ~"std-in" )) {
            fail ~"could not insert cmd into json command";
          }
          if (!cmd_map.insert( ~"payload", payload )) {
            fail ~"could not insert payload into json command";
          };

          let bridge_cmd = cmd_map.to_json().to_str();
          da_socket.write( core::str::to_bytes(bridge_cmd) );
        }
      };

    }
  }
}


#[allow(non_implicitly_copyable_typarams)]
fn main() {
  let (pid_file, input_file, bridge_cmd) = parse_cmd_arguments();
  let props = ~std::map::HashMap();

  wol::property_file::read_file(props, input_file);

  let contains_jar        =  props.find(~"jar");
  let contains_classpath  =  props.find(~"classpath");
  let contains_main_class =  props.find(~"main.class");

  let bridge : daemon_tools::Bridge = match (contains_jar, contains_classpath, contains_main_class) {
    (None,   None    , None)      => { fail(~"jar or classpath + main class must be specified in properties file.  Neither load strategy found"); }
    (Some(_), Some(_), Some(_))   => { fail(~"jar AND classpath + main class specified in properties file. Please pick a single load strategy"); }
    (Some(_), Some(_), None)      => { fail(~"jar AND classpath + main class specified in properties file. Please pick a single load strategy"); }
    (Some(_), None,    Some(_))   => { fail(~"jar AND classpath + main class specified in properties file. Please pick a single load strategy"); }
    (Some(r), None, None)         => { daemon_tools::Bridge { strategy: daemon_tools::JarStrategy(r),
                                                              pid_file : pid_file } }

    (None, None,   Some(_))       => { fail(~"main class specified but not classpath") }
    (None, Some(_), None)         => { fail(~"classpath specified but not main class") }
    (None, Some(r), Some(s))      => { daemon_tools::Bridge { strategy : daemon_tools::ClassPathStrategy(r,s),
                                                              pid_file : pid_file } }
  };


  //wol::property_file::print_properties(props);

  let socket : std::net_tcp::TcpSocket = ensure_connection( props.get(~"host"), props.get(~"port"), &bridge);
  //let socket_buff : std::net_tcp::TcpSocketBuf = std::net_tcp::socket_buf(move socket);

  let mut cmd_map : LinearMap<~str,~str> = LinearMap();
  if (!cmd_map.insert( ~"cmd", ~"exec" )) {
    fail ~"could not insert cmd into json command";
  }
  if (!cmd_map.insert( ~"payload", bridge_cmd)) {
    fail ~"could not insert payload into json command";
  };
  let bridge_cmd_json = cmd_map.to_json().to_str();

  //socket_buff.write( core::str::to_bytes(bridge_cmd_json) );
  socket.write( core::str::to_bytes(bridge_cmd_json) );

  let (std_out_port, std_out_chan): (Port<Option<~str>>, Chan<Option<~str>>) = stream();
  let (std_err_port, std_err_chan): (Port<Option<~str>>, Chan<Option<~str>>) = stream();
  let (std_in_port, std_in_chan):   (Port<Option<~str>>, Chan<Option<~str>>) = stream();

  do spawn |move std_err_port| {
    loop {
      let cmd = std_err_port.recv();
      match cmd {
        None => { break; }
        Some(payload) => {  io::stderr().write_str(payload);
          io::stderr().flush(); }
      };
    }
  }

  do spawn |move std_out_port| {
    loop {
      let cmd = std_out_port.recv();
      match cmd {
        None => { break; }
        Some(payload) => {  io::print(payload);
          io::stdout().flush(); }
      };
    }
  }

  do spawn |move std_in_chan| {
    loop {
      if (io::stdin().eof()) {
        io::println("stdin EOF reached");
        break;
      }
      let next_line = io::stdin().read_line();
      std_in_chan.send( Some( str::append( copy next_line,  ~"\n")) );
    }
  }

  event_loop(socket, std_out_chan, std_err_chan, std_in_port);
  //parse_cmd(socket_buff, std_out_chan, std_err_chan);
}

