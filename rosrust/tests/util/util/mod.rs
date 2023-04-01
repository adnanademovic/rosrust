pub use child_process_terminator::ChildProcessTerminator;
use std::env;
use std::process::{Command, Output};
use std::str::from_utf8;
use std::thread::sleep;
use std::time::Duration;
pub use subscriber_test::{test_publisher, test_subscriber, test_subscriber_detailed};
pub use test_variant::TestVariant;

mod child_process_terminator;
mod subscriber_test;
mod test_variant;

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

fn run_roscore(port: u32) -> ChildProcessTerminator {
    println!("Running roscore on port: {}", port);
    env::set_var("ROS_MASTER_URI", format!("http://localhost:{}", port));
    let roscore =
        ChildProcessTerminator::spawn(Command::new("roscore").arg("-p").arg(format!("{}", port)));
    await_roscore();
    roscore
}

pub fn run_roscore_for(test_variant: TestVariant) -> ChildProcessTerminator {
    run_roscore(test_variant.port())
}

pub fn bytes_contain(sequence: &[u8], subsequence: &[u8]) -> bool {
    sequence
        .windows(subsequence.len())
        .any(|window| window == subsequence)
}

#[allow(dead_code)]
pub fn assert_success_and_output_containing(output: Output, expected: &str) {
    assert!(
        output.status.success(),
        "STDERR: {}",
        from_utf8(&output.stderr).unwrap_or("not valid UTF-8"),
    );
    let stdout = output.stdout;
    assert!(
        bytes_contain(&stdout, expected.as_bytes()),
        "expected: {}, STDOUT: {}",
        expected,
        from_utf8(&stdout).unwrap_or("not valid UTF-8")
    );
}
