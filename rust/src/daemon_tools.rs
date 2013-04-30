pub struct Bridge { 
  pid_file : path::Path,
  strategy : LoadStrategy 
}

pub enum LoadStrategy {
  JarStrategy(~str),
  ClassPathStrategy(~str,~str) 
}

fn run_cp_strategy(cp: &~str, main_class: &~str) -> () {
  libc::funcs::posix88::unistd::setsid();
  do str::as_c_str(~"java") |c_cmd| {
    do str::as_c_str(~"-cp") |c_cp_flag| {
      do str::as_c_str(*cp) |c_cp| {
        do str::as_c_str(~"clojure.main") |c_clj_class| {
          do str::as_c_str(~"-m") |c_main_flag| {
            do str::as_c_str(*main_class) |c_namespace| {
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

fn run_jar_strategy(jar: &~str) -> () {
  //NB> check for EPERM setsid failure
  libc::funcs::posix88::unistd::setsid();
  libc::funcs::posix88::unistd::close(libc::consts::os::posix88::STDIN_FILENO as core::libc::types::os::arch::c95::c_int);
  libc::funcs::posix88::unistd::close(libc::consts::os::posix88::STDOUT_FILENO as core::libc::types::os::arch::c95::c_int);
  libc::funcs::posix88::unistd::close(libc::consts::os::posix88::STDERR_FILENO as core::libc::types::os::arch::c95::c_int);
  do str::as_c_str(~"java") |c_cmd| {
    do str::as_c_str(~"-jar") |c_jar_flag| {
      do str::as_c_str(*jar) |c_jar_location| {
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

fn produce_pid_file(target_path : &path::Path ) -> () {
  let our_pid = libc::funcs::posix88::unistd::getpid();

  let w: Result<io::Writer,~str> = io::buffered_file_writer(target_path); 
  if w.is_err() {
    io::println(~"unable to open pid_file.txt for writing");
    return;
  }

  let wrt: io::Writer = w.get();
  wrt.write_int(our_pid as int);
}

pub fn daemonize(bridge : &Bridge ) -> () {
  produce_pid_file(&(*bridge).pid_file);
  let strategy = &bridge.strategy;

  match *strategy {
    JarStrategy(ref location) => { run_jar_strategy(location) }
    ClassPathStrategy(ref cp, ref main_class) => { run_cp_strategy(cp,main_class) }
  }
}



