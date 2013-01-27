extern mod std;
use std::map;
use libc::{c_char, c_int, size_t};
//use io::ReaderUtil;


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

priv fn open_stream(input_file: ~str) -> *libc::types::common::c95::FILE {
  do str::as_c_str(input_file) |file_name|  {
    do str::as_c_str("r") |file_mode| {
      libc::funcs::c95::stdio::fopen(file_name , file_mode)
    }
  }
}

pub fn read_file(input_file: ~str) -> std::map::HashMap<@~str,@~str> {
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

  let stream = open_stream( input_file );
  let mut finished_reading : bool = false;
  let mut next_line : *libc::c_char;

  if ptr::is_null(stream)  {
    fail #fmt("Error: Couldn't locate config file: %s", input_file);
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

