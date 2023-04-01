use rosrust_diagnostics::{Level, Status, Task, TimestampStatus};
mod util;

#[test]
fn timestamp_status_test() {
    let _roscore = util::run_roscore_for(util::TestVariant::TimestampStatusTest);
    rosrust::init("timestamp_status_test");
    let ts = TimestampStatus::builder().build();
    let mut status0 = Status::default();
    ts.run(&mut status0);
    ts.tick_float(rosrust::now().seconds() + 2.0);
    let mut status1 = Status::default();
    ts.run(&mut status1);
    ts.tick(rosrust::now());
    let mut status2 = Status::default();
    ts.run(&mut status2);
    ts.tick_float(rosrust::now().seconds() - 4.0);
    let mut status3 = Status::default();
    ts.run(&mut status3);
    ts.tick_float(rosrust::now().seconds() - 6.0);
    let mut status4 = Status::default();
    ts.run(&mut status4);

    assert_eq!(
        status0.level,
        Level::Warn,
        "no data should return a warning"
    );
    assert_eq!(status1.level, Level::Error, "too far future not reported");
    assert_eq!(status2.level, Level::Ok, "now not accepted");
    assert_eq!(status3.level, Level::Ok, "4 seconds ago not accepted");
    assert_eq!(status4.level, Level::Error, "too far past not reported");
    assert_eq!(
        status0.name, "",
        "Name should not be set by TimeStapmStatus"
    );
    assert_eq!(
        ts.name(),
        "Timestamp Status",
        "Name should be \"Timestamp Status\""
    );
}
