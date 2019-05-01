use rosrust::Duration;
use rosrust_diagnostics::{FrequencyStatus, Level, Status, Task};

mod util;

#[test]
fn frequency_status_test() {
    let _roscore = util::run_roscore_for(util::Feature::FrequencyStatusTest);
    rosrust::init("frequency_status_test");

    let fs = FrequencyStatus::builder()
        .window_size(2)
        .min_frequency(10.0)
        .max_frequency(20.0)
        .tolerance(0.5)
        .build();

    fs.tick();
    rosrust::sleep(Duration::from_nanos(20_000_000));
    let mut status0 = Status::default();
    fs.run(&mut status0);
    rosrust::sleep(Duration::from_nanos(50_000_000));
    fs.tick();
    let mut status1 = Status::default();
    fs.run(&mut status1);
    rosrust::sleep(Duration::from_nanos(300_000_000));
    fs.tick();
    let mut status2 = Status::default();
    fs.run(&mut status2);
    rosrust::sleep(Duration::from_nanos(150_000_000));
    fs.tick();
    let mut status3 = Status::default();
    fs.run(&mut status3);
    fs.clear();
    let mut status4 = Status::default();
    fs.run(&mut status4);

    assert_eq!(
        status0.level,
        Level::Warn,
        "Max frequency exceeded but not reported"
    );
    assert_eq!(
        status1.level,
        Level::Ok,
        "Within max frequency but reported error"
    );
    assert_eq!(
        status2.level,
        Level::Ok,
        "Within min frequency but reported error"
    );
    assert_eq!(
        status3.level,
        Level::Warn,
        "Min frequency exceeded but not reported"
    );
    assert_eq!(status4.level, Level::Error, "Freshly cleared should fail");
    assert_eq!(
        status0.name, "",
        "Name should not be set by FrequencyStatus"
    );
    assert_eq!(
        fs.name(),
        "Frequency Status",
        "Name should be \"Frequency Status\""
    );
}
