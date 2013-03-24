
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

use io::WriterUtil;

enum LoadStrategy {
  JarStrategy(~str),
  ClassPathStrategy(~str,~str) 
}

fn run_cp_strategy(cp: ~str, main_class: ~str) -> () {
  libc::funcs::posix88::unistd::setsid();
  do str::as_c_str(~"java") |c_cmd| {
    do str::as_c_str(~"-cp") |c_cp_flag| {
      do str::as_c_str(cp) |c_cp| {
        do str::as_c_str(~"clojure.main") |c_clj_class| {
          do str::as_c_str(~"-m") |c_main_flag| {
            do str::as_c_str(main_class) |c_namespace| {
              do str::as_c_str(~"--server") |c_server_flag| {
                let null_ptr  = ptr::null();
                let args      = [c_cmd, c_cp_flag, c_cp, c_clj_class, c_main_flag, c_namespace, c_server_flag, null_ptr];
                unsafe {
                  let result = libc::funcs::posix88::unistd::execvp( c_cmd,  vec::raw::to_ptr(args) );  
                  io::println(result.to_str());
                  if (result == -1) {
                    fail(~"unable to exec jvm");
                  }
                }
              }
            }
          }
        }
      }
    }
  }
}

fn run_jar_strategy(jar: ~str) -> () {
  //NB> check for EPERM setsid failure
  libc::funcs::posix88::unistd::setsid();
  do str::as_c_str(~"java") |c_cmd| {
    do str::as_c_str(~"-jar") |c_jar_flag| {
      do str::as_c_str(jar) |c_jar_location| {
        do str::as_c_str(~"--server") |c_server_flag| {
          let null_ptr  = ptr::null();
          let args      = [c_cmd, c_jar_flag, c_jar_location, c_server_flag, null_ptr];
          unsafe {
            let result = libc::funcs::posix88::unistd::execvp( c_cmd,  vec::raw::to_ptr(args) );  
            io::println(result.to_str());
            if (result == -1) {
              fail(~"unable to exec jvm");
            }
          }
        }
      }
    }
  }
}

fn produce_pid_file() -> () {
  let our_pid = libc::funcs::posix88::unistd::getpid();

  let w: Result<io::Writer,~str> = io::buffered_file_writer(&path::Path("pid_file.txt")); 
  if w.is_err() {
    io::println(~"unable to open pid_file.txt for writing");
    return;
  }

  let wrt: io::Writer = w.get();
  wrt.write_int(our_pid as int);
}

fn daemonize(strategy: LoadStrategy) -> () {
  produce_pid_file();

  match strategy {
    JarStrategy(location) => { run_jar_strategy(location) }
    ClassPathStrategy(cp,main_class) => { run_cp_strategy(cp,main_class) }
  }
}


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

fn ensure_connection(host: ~str, port: ~str, strategy: LoadStrategy) -> (std::net_tcp::TcpSocket) {
  let io_task = uv::global_loop::get();
  let conn_res : Result<std::net_tcp::TcpSocket,std::net_tcp::TcpConnectErrData> = std::net_tcp::connect( std::net_ip::v4::parse_addr(host.to_managed()), 
      option::unwrap(uint::from_str(port.to_managed())), 
      io_task);
  if conn_res.is_err() {
    let pid = libc::funcs::posix88::unistd::fork();
    if (pid < 0) {
      fail ~"Error: unable to fork whilst trying to launch jvm"
    } else if (pid == 0) {
      daemonize(strategy);
    } else {
      return poll_for_connection(host,port);
    }
  } 
  core::result::unwrap(conn_res)
}

fn parse_cmd_arguments() -> (~str,~str) {
  let opts = ~[ optopt("c") ];
  let args = os::args();

  let matches = match getopts(vec::view(args, 1, args.len()), opts) {
    result::Ok(m)  => { copy m }
    result::Err(f) => { fail fail_str(copy f) }
  };

  let input_file = match opt_maybe_str(copy &matches, "c" ) {
    option::Some(s) => { copy s }
    option::None() => { core::os::getcwd().push(".rusted-bridge").to_str() } 

  };

  if (vec::is_empty(matches.free)) {
    fail ~"please specify a command to send over the bridge"
  }

  let bridge_cmd = matches.free.foldl( ~"", |accum,e| str::append(copy *accum, str::append( ~"/",  *e) ) );

  return (input_file, bridge_cmd);
}



#[allow(non_implicitly_copyable_typarams)]
fn parse_cmd_2(next_cmd : &str) -> (Option<(~Object,~str)>) {
  let mut idx = 1;
  while (idx < next_cmd.len()) {
    let framed_cmd = core::str::slice(next_cmd,0,idx);
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

//
#[allow(non_implicitly_copyable_typarams)]
fn event_loop(da_socket : std::net_tcp::TcpSocket, 
    std_out_channel : Chan<Option<~str>>,
    std_err_channel : Chan<Option<~str>>) -> () {

  let mut next_cmd : ~str = ~"";
  let result = std::net_tcp::read_start( &da_socket );

  let socket_port = match(result.is_err()) {
    false => { core::result::unwrap(result) }
    true  => { fail(~"Erro reading socket"); }
  };

  loop {
    if (socket_port.peek()) {
      let next_result = socket_port.recv();
      let next_read = match (next_result.is_err())  {
        true => { 
         let err_data = result::unwrap_err(next_result);
         if err_data.err_name == ~"EOF" {
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
        let (json_response,remaining_cmd) = match(parse_cmd_2(next_cmd)) {
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
  }
}

#[allow(non_implicitly_copyable_typarams)]
fn parse_cmd(socket_buff : std::net_tcp::TcpSocketBuf, std_out_channel : Chan<Option<~str>>,
    std_err_channel : Chan<Option<~str>>) -> () {
  let mut next_cmd : ~str = ~"";
  loop {
    //libc::funcs::posix88::unistd::sleep(1);
    let read_res = socket_buff.read_byte();

    //socket connection was closed
    if (socket_buff.eof()) {
      std_out_channel.send(None);
      std_err_channel.send(None);
      break;  
    }

    let next_char : ~str = core::str::from_byte(read_res as u8);
    next_cmd = core::str::append(next_cmd, next_char);

    //io::println(fmt!("next_cmd : %s", next_cmd ));

    let parse_result : Result<Json,Error> = std::json::from_str(next_cmd);

    if (!parse_result.is_err()) {
      //io::println(fmt!("successfully parsed :%s",next_cmd));
      let json_response = match( core::result::unwrap(parse_result) ) {
        Object(cmd) => { cmd }
        _           => { fail(~"received command was wrong type"); }
      };

      let cmd_str = match (json_response.find(&~"command")) {
        None      => { fail(~"response malformed. no command found") }
        Some(cmd) =>  { Decoder(cmd).read_owned_str() } 
      };
      //io::println(fmt!("cmd: %s", cmd_str));  

      let payload_str = match (json_response.find(&~"payload")) {
        None => { fail(~"response malformed. no payload found") }
        Some(payload) =>  { Decoder(payload).read_owned_str() } 
      };

      match (cmd_str) {
        ~"std-out" => { std_out_channel.send(Some(payload_str)); }
        ~"std-err" => { std_err_channel.send(Some(payload_str)); }
        _          => { fail(fmt!("unrecognized command %s", cmd_str)); }
      }
      next_cmd = ~"";
    }
  }
}

#[allow(non_implicitly_copyable_typarams)]
fn main() {
  let (input_file,bridge_cmd) = parse_cmd_arguments();
  let props = ~std::map::HashMap();

  wol::property_file::read_file(props, input_file);

  let contains_jar        =  props.find(~"jar");
  let contains_classpath  =  props.find(~"classpath");
  let contains_main_class =  props.find(~"main.class");

  let strategy = match (contains_jar, contains_classpath, contains_main_class) {
    (None,   None    , None)      => { fail(~"jar or classpath + main class must be specified in properties file.  Neither load strategy found"); }
    (Some(_), Some(_), Some(_))   => { fail(~"jar AND classpath + main class specified in properties file. Please pick a single load strategy"); }
    (Some(_), Some(_), None)      => { fail(~"jar AND classpath + main class specified in properties file. Please pick a single load strategy"); }
    (Some(_), None,    Some(_))   => { fail(~"jar AND classpath + main class specified in properties file. Please pick a single load strategy"); }
    (Some(r), None, None)         => { JarStrategy(r) }

    (None, None,   Some(_))       => { fail(~"main class specified but not classpath") }
    (None, Some(_), None)         => { fail(~"classpath specified but not main class") }
    (None, Some(r), Some(s))      => { ClassPathStrategy(r,s) }
  };


  //wol::property_file::print_properties(props);

  let socket : std::net_tcp::TcpSocket = ensure_connection( props.get(~"host"), props.get(~"port"), strategy);
  //let socket_buff : std::net_tcp::TcpSocketBuf = std::net_tcp::socket_buf(move socket);

  //io::println("connection established!");
  let bridge_cmd_json = bridge_cmd.to_json().to_str();

  //socket_buff.write( core::str::to_bytes(bridge_cmd_json) );
  socket.write( core::str::to_bytes(bridge_cmd_json) );

  let (std_out_port, std_out_chan): (Port<Option<~str>>, Chan<Option<~str>>) = stream();
  let (std_err_port, std_err_chan): (Port<Option<~str>>, Chan<Option<~str>>) = stream();

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

  event_loop(socket, std_out_chan, std_err_chan);
  //parse_cmd(socket_buff, std_out_chan, std_err_chan);
}

