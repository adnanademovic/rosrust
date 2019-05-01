use nix::sys::signal::{kill, Signal};
use nix::unistd::Pid;
use std::process::{Child, Command, Stdio};

#[must_use]
pub struct ChildProcessTerminator(pub Child);

impl ChildProcessTerminator {
    pub fn spawn(command: &mut Command) -> ChildProcessTerminator {
        command
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null());
        ChildProcessTerminator(command.spawn().unwrap())
    }
}

impl Drop for ChildProcessTerminator {
    fn drop(&mut self) {
        let pid = Pid::from_raw(self.0.id() as i32);
        kill(pid, Signal::SIGINT).unwrap();
    }
}
