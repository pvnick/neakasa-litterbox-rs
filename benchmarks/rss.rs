//! Small Linux launcher: avoid Python's resident memory contaminating child RSS.
use std::{
    io,
    process::{Command, Stdio},
};

// wait4 below reaps the child and supplies resource usage, which Child::wait omits.
#[allow(clippy::zombie_processes)]
fn main() {
    let mut args = std::env::args().skip(1);
    let mut command = Command::new(args.next().expect("executable required"));
    command
        .args(args)
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    let child = command.spawn().expect("launch benchmark CLI");
    let mut status = 0;
    // SAFETY: rusage is a C struct of integers; zero is valid for every field.
    let mut usage: libc::rusage = unsafe { std::mem::zeroed() };
    loop {
        // SAFETY: both output pointers are valid and the PID belongs to our child.
        let pid = unsafe { libc::wait4(child.id() as libc::pid_t, &mut status, 0, &mut usage) };
        if pid >= 0 {
            break;
        }
        assert_eq!(
            io::Error::last_os_error().kind(),
            io::ErrorKind::Interrupted
        );
    }
    assert!(
        libc::WIFEXITED(status) && libc::WEXITSTATUS(status) == 0,
        "CLI failed"
    );
    println!("{}", usage.ru_maxrss);
}
