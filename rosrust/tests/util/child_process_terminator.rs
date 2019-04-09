use nix::sys::signal::{kill, Signal};
use nix::unistd::Pid;
use std::fs::canonicalize;
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

    #[allow(dead_code)]
    pub fn spawn_example(path: &str, command: &mut Command) -> ChildProcessTerminator {
        let canonical_path = canonicalize(path).unwrap();
        assert!(Command::new("cargo")
            .arg("build")
            .current_dir(&canonical_path)
            .output()
            .unwrap()
            .status
            .success());

        Self::spawn(command.current_dir(&canonical_path))
    }

    #[allow(dead_code)]
    pub fn spawn_example_bench(path: &str, command: &mut Command) -> ChildProcessTerminator {
        let canonical_path = canonicalize(path).unwrap();
        assert!(Command::new("cargo")
            .arg("build")
            .arg("--release")
            .current_dir(&canonical_path)
            .output()
            .unwrap()
            .status
            .success());

        Self::spawn(command.current_dir(&canonical_path))
    }
}

impl Drop for ChildProcessTerminator {
    fn drop(&mut self) {
        let pid = Pid::from_raw(self.0.id() as i32);
        kill(pid, Signal::SIGINT).unwrap();
    }
}
