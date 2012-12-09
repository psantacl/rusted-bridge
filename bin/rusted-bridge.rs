extern mod std;
use std::map;

use libc::{c_char, c_int, size_t};

mod property_file {

    priv fn process_line( line : *libc::types::os::arch::c95::c_char, properties : std::map::HashMap<@~str,@~str>) -> ()  {
        do str::as_c_str("=") |c_equals| {
            unsafe {
                let k_token = libc::funcs::c95::string::strtok(line, c_equals);
                //line is not a valid property(key=value)
                if ptr::is_null(k_token) {
                    libc::funcs::c95::stdlib::free(line as *core::libc::types::common::c95::c_void);
                } else {
                    let v_token =  libc::funcs::c95::string::strtok( ptr::null(), c_equals);
                    //remove trailing newline
                    libc::funcs::c95::string::memset(ptr::offset(v_token, 
                                                                (libc::funcs::c95::string::strlen(v_token) - 1) as uint) as *libc::c_void, 
                                                                0,
                                                                1);
                    
                    properties.insert(@str::raw::from_c_str(k_token),
                                      @str::raw::from_c_str(v_token));
                }
            }
        }
    }

    priv fn read_line(file_stream : *libc::types::common::c95::FILE) -> (*libc::c_char, bool) {
        let read_block_increment : int = 2;
        let mut read_block_size  : int = read_block_increment;
        let mut line_read        : bool = false;
        let mut finished_reading : bool = false;
        let c_newline = str::as_c_str("\n", { |burger| burger });
        let mut read_buffer = libc::funcs::c95::stdlib::malloc(read_block_size as core::libc::types::os::arch::c95::size_t);

        if ptr::is_null(read_buffer)  {
          fail(#fmt("failed to allocated read buffer of size %d", read_block_size));
        }

        let file_position = libc::funcs::c95::stdio::ftell(file_stream);

        while !line_read {
            unsafe {
                let next_chunk = libc::funcs::c95::stdio::fgets(read_buffer as *mut libc::c_char,
                                                                read_block_size as libc::c_int,
                                                                file_stream);
                //eof encountered, no bytes read
                if ptr::is_null(next_chunk)  {
                    finished_reading = true;
                    break;
                }
                //eof encountered, with bytes read
                if libc::funcs::c95::stdio::feof(file_stream) != 0 {
                    finished_reading = true;  
                    break;
                }

                let nl_char = libc::funcs::c95::string::strchr(next_chunk, *c_newline as libc::c_int);

                if ptr::is_null(nl_char) {
                    read_block_size += read_block_increment;

                    let new_buffer = libc::funcs::c95::stdlib::realloc(read_buffer, read_block_size as libc::size_t);
          
                    if ptr::is_null(new_buffer)  {
                      fail(#fmt("failed to realloc read buffer to size %d", read_block_size));
                    }

                    read_buffer = new_buffer;

                    libc::funcs::c95::stdio::fseek( file_stream, file_position, 0);
                } else {
                    line_read = true;
                }
            }
        }
        return (read_buffer as *libc::c_char, finished_reading);
   } 

    pub fn read_file(p: Path) -> std::map::HashMap<@~str,@~str> {
        let properties = std::map::HashMap();
        //let r: Result<io::Reader,~str> = io::file_reader(&p); // r is result<reader, err_str>
        //if r.is_err() {
        //    fail result::unwrap_err(r);
        //}

        //let rdr: io::Reader = r.get();
        //while !rdr.eof() {
        //    let nextLine: ~str = rdr.read_line();
        //    //let data:@str = match nextLine { ~copy data => @data };
        //    //let data:@str = match nextLine { data => move @*data };
        //    //let data = str::split_char(nextLine, '=');
        //    //properties.insert(key,value);
        //    io::println(nextLine);
        //}

        let file_name = str::as_c_str(p.to_str(), { |file_name| file_name });
        let file_mode = str::as_c_str("r", { |file_mode| file_mode });
        let stream = libc::funcs::c95::stdio::fopen(file_name , file_mode);
        let mut finished_reading : bool = false;
        let mut next_line : *libc::c_char;

        if ptr::is_null(stream)  {
            fail #fmt("Error: Couldn't locate config file: %s", p.to_str());
        }

        while !finished_reading {
            match read_line(stream) {
                (x,y) => {
                    next_line = x;
                    finished_reading = y;
                }
            }
            if !ptr::is_null(next_line) {
                process_line(next_line, properties);
            }
        }

        return properties;
    }
}

use io::ReaderUtil;
use property_file::read_file;
use std::uv_iotask;
use std::net_tcp;
use std::net_ip::v4;
use std::uv;
use str::raw::from_c_str;

fn main() {
    //let args: ~[~str] = os::args();
    //if args.len() == 1 {
    //    fail #fmt("usage: %s <filename>", args[0]);
    //}
    //let p: Path = path::Path(args[1]);

    let p: Path = core::os::getcwd().push(".rusted-bridge");

    let props = property_file::read_file(p);

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
      
    io::println(#fmt("port is %s", *props.get(@~"port")));

    let io_task = uv::global_loop::get();
    let conn_res : Result<std::net_tcp::TcpSocket,std::net_tcp::TcpConnectErrData> = std::net_tcp::connect(std::net_ip::v4::parse_addr(*props.get(@~"host")), 
                                                                                                           option::unwrap(uint::from_str(*props.get(@~"port"))),
                                                                                                           io_task); 
    if conn_res.is_err() {
        fail ~"failed to connect to socket"
    }
    let socket_conn = core::result::unwrap(conn_res);
    let write_res = socket_conn.write(core::str::to_bytes("is there anybody there in this electronic void?"));
    if write_res.is_err() {
      fail ~"error sending command over socket"
    }

    loop {
      let read_res = socket_conn.read(0u);
      if read_res.is_err()  {
        //let err_data = read_res.get_err();
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
