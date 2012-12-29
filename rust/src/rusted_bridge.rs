extern mod std;
extern mod wol;

use wol::property_file::read_file;
use std::uv_iotask;
use std::net_tcp;
use std::net_ip::v4;
use std::uv;
use str::raw::from_c_str;

use core::result::{Ok,Err};
use std::getopts::{optopt, getopts, opt_maybe_str, fail_str };
use std::json::{ToJson};

fn parse_cmd_arguments() -> (~str,~str) {
  let opts = ~[ optopt("c") ];
  let args = os::args();
  //enum Shape {
  //  Circle(int),
  //  Rectangle(float)
  //};
  //let chicken = Rectangle(4.0);
  //match chicken {
  //  Circle(_) => unsafe { io::println("its a circle") },
  //  Rectangle(_) => unsafe { io::println("its a square") }
  //}

  let matches = match getopts(vec::view(args, 1, args.len()), opts) {
    result::Ok(m)  => { copy m }
    result::Err(f) => { fail fail_str(copy f) }
   };

  let input_file = match opt_maybe_str(copy matches, "c" ) {
    option::Some(s) => { copy s }
    option::None() => { core::os::getcwd().push(".rusted-bridge").to_str() } 

  };
  
  if (vec::is_empty(matches.free)) {
    fail ~"please specify a command to send over the bridge"
  }

  let bridge_cmd = matches.free.foldl( ~"", |accum,e| str::append(copy *accum, str::append( ~"/",  *e) ) );

  return (input_file, bridge_cmd);
}


fn main() {
    //let args: ~[~str] = os::args();
    //if args.len() == 1 {
    //    fail #fmt("usage: %s <filename>", args[0]);
    //}
    //let p: Path = path::Path(args[1]);

    let (input_file,bridge_cmd) = parse_cmd_arguments();
    //io::println( #fmt("input file: %s", input_file )); 
    //io::println( #fmt("cmd: %s", bridge_cmd)); 

    let props = wol::property_file::read_file(input_file);

    //for props.each |k,v| {
    //    unsafe {
    //        io::print(k);
    //        io::print(" -> ");
    //        io::print(v);
    //        io::println("");
    //    }
    //}

    //io::println(#fmt("connecting to port %s", port));
    //let our_task : core::task::TaskBuilder = core::task::task();
    //let our_io_task : std::uv_iotask::IoTask  = std::uv_iotask::spawn_iotask(our_task);
      
    //io::println(#fmt("port: %s", *props.get(@~"port")));

    let io_task = uv::global_loop::get();
    let conn_res : Result<std::net_tcp::TcpSocket,std::net_tcp::TcpConnectErrData> = std::net_tcp::connect(std::net_ip::v4::parse_addr(*props.get(@~"host")), 
                                                                                                           option::unwrap(uint::from_str(*props.get(@~"port"))),
                                                                                                           io_task); 
    if conn_res.is_err() {
        fail ~"failed to connect to socket"
    }
    let socket_conn = core::result::unwrap(conn_res);
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
