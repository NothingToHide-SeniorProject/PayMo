use nix::{
    sys::wait::waitpid,
    unistd::{fork, write, ForkResult},
};
use std::env;

fn main() {
    match unsafe { fork() } {
        Ok(ForkResult::Parent { child }) => {
            println!("Continuing execution in parent process, new child has pid: {child}");
            waitpid(child, None);
        }
        Ok(ForkResult::Child) => {
            write(libc::STDOUT_FILENO, "I'm a new child process\n".as_bytes()).ok();
            std::process::exit(0);

            // unsafe { libc::_exit(0) };
        }
        Err(_) => println!("Fork failed"),
    }
}
