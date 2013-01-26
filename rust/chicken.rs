use libc::{c_char};

fn main () {
  let null_ptr  = ptr::null();
  do str::as_c_str("ls") |c_ls_path| {
      do str::as_c_str("ls") |c_ls| {
          do str::as_c_str("-l") |c_ls_flag| {
              let args     = [c_ls, c_ls_flag, null_ptr];
              unsafe {
                let result = libc::funcs::posix88::unistd::execvp( c_ls_path,  vec::raw::to_ptr(args) );  
                io::println(result.to_str());
              }
          }
      }
  }

}
//fn main () {
//
//  let null_ptr = ptr::null();
//  let c_ls     = str::as_c_str(~"ls", { |ls| ls });
//  let c_ls_arg = str::as_c_str(~"-l", { |ls| ls });
//
//  let args     = [c_ls, c_ls_arg, null_ptr];
//  unsafe {
//    let result = libc::funcs::posix88::unistd::execvp( c_ls,  vec::raw::to_ptr(args) ); 
//    io::println(core::i32::to_str(result,10));
//  }
//}
