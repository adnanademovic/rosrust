use crate::{Duration, Time};
use std::time;

#[test]
fn from_nanos_works() {
    let time = Time::from_nanos(123456789987654321);
    assert_eq!(time.sec, 123456789);
    assert_eq!(time.nsec, 987654321);
    let time = Duration::from_nanos(123456789987654321);
    assert_eq!(time.sec, 123456789);
    assert_eq!(time.nsec, 987654321);
}

#[test]
fn nanos_works() {
    let time = Time {
        sec: 123456789,
        nsec: 987654321,
    };
    assert_eq!(time.nanos(), 123456789987654321);
    let time = Duration {
        sec: 123456789,
        nsec: 987654321,
    };
    assert_eq!(time.nanos(), 123456789987654321);
}

#[test]
fn duration_works_with_negative() {
    let time = Duration::from_nanos(-123456789987654321);
    assert_eq!(time.sec, -123456789);
    assert_eq!(time.nsec, -987654321);
    assert_eq!(time.nanos(), -123456789987654321);
}

#[test]
fn convert_works() {
    let std_duration = time::Duration::new(123, 456);
    let msg_duration = Duration::from(std_duration);
    assert_eq!(msg_duration.sec, 123);
    assert_eq!(msg_duration.nsec, 456);

    let std_duration2 = time::Duration::new(9876, 54321);
    let msg_duration2: Duration = std_duration2.into();
    assert_eq!(msg_duration2.sec, 9876);
    assert_eq!(msg_duration2.nsec, 54321);
}
