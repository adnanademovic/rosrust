use rosrust_diagnostics::{Heartbeat, Level, Status, Task};

#[test]
fn heartbeat_test() {
    let hb = Heartbeat;

    let mut status = Status::default();
    hb.run(&mut status);

    assert_eq!(
        status.level,
        Level::Ok,
        "Heartbeat did not return an OK status"
    );
    assert_eq!(status.name, "", "Name should not be set by FrequencyStatus");
    assert!(
        status.values.is_empty(),
        "Heartbeat should not set any values"
    );
    assert_eq!(hb.name(), "Heartbeat", "Name should be \"Heartbeat\"");
}
