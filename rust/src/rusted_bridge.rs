
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
use std::json::{ToJson};
use libc::{c_char};

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
    let conn_res : Result<std::net_tcp::TcpSocket,std::net_tcp::TcpConnectErrData> = std::net_tcp::connect(std::net_ip::v4::parse_addr(host.to_managed()), 
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

    let socket_conn = ensure_connection( props.get(~"host"), props.get(~"port"), strategy);
    //io::println("connection established!");
    let bridge_cmd_json = bridge_cmd.to_json().to_str();

    let write_res = socket_conn.write( core::str::to_bytes(bridge_cmd_json) );
    if write_res.is_err() {
      fail ~"error sending command over socket"
    }

    loop {
      let read_res = socket_conn.read(0u);
      if read_res.is_err()  {
        let err_data = result::unwrap_err(read_res);
        if err_data.err_name == ~"EOF" {
          break;
        } else {
          fail ~"error getting response"
        }
      }

      io::println(core::str::from_bytes(core::result::unwrap(read_res)));
    }
  }

