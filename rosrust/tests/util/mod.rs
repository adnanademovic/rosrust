pub use child_process_terminator::ChildProcessTerminator;
use std::env;
use std::process::Command;
use std::thread::sleep;
use std::time::Duration;

mod child_process_terminator;

fn rostopic_listing_succeeds() -> bool {
    return Command::new("rostopic")
        .arg("list")
        .output()
        .unwrap()
        .status
        .success();
}

fn await_roscore() {
    while !rostopic_listing_succeeds() {
        sleep(Duration::from_millis(100));
    }
}

pub fn run_roscore(port: u32) -> ChildProcessTerminator {
    env::set_var("ROS_MASTER_URI", format!("http://localhost:{}", port));
    let roscore = ChildProcessTerminator::spawn(
        &mut Command::new("roscore").arg("-p").arg(format!("{}", port)),
    );
    await_roscore();
    roscore
}
