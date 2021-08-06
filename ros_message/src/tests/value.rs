use crate::{MessageValue, Time, Value};

#[test]
fn display() {
    let mut header = MessageValue::new();
    header.insert("seq".into(), Value::U32(22));
    header.insert(
        "stamp".into(),
        Value::Time(Time {
            sec: 123,
            nsec: 100_000_000,
        }),
    );
    header.insert("frame_id".into(), Value::String("abc".into()));

    let mut position1 = MessageValue::new();
    position1.insert("x".into(), Value::F64(1.0));
    position1.insert("y".into(), Value::F64(2.0));
    position1.insert("z".into(), Value::F64(3.0));

    let mut orientation1 = MessageValue::new();
    orientation1.insert("x".into(), Value::F64(4.0));
    orientation1.insert("y".into(), Value::F64(5.0));
    orientation1.insert("z".into(), Value::F64(6.0));
    orientation1.insert("w".into(), Value::F64(7.0));

    let mut pose1 = MessageValue::new();
    pose1.insert("position".into(), Value::Message(position1));
    pose1.insert("orientation".into(), Value::Message(orientation1));

    let mut position2 = MessageValue::new();
    position2.insert("x".into(), Value::F64(8.0));
    position2.insert("y".into(), Value::F64(9.0));
    position2.insert("z".into(), Value::F64(10.0));

    let mut orientation2 = MessageValue::new();
    orientation2.insert("x".into(), Value::F64(11.0));
    orientation2.insert("y".into(), Value::F64(12.0));
    orientation2.insert("z".into(), Value::F64(13.0));
    orientation2.insert("w".into(), Value::F64(14.0));

    let mut pose2 = MessageValue::new();
    pose2.insert("position".into(), Value::Message(position2));
    pose2.insert("orientation".into(), Value::Message(orientation2));

    let poses = vec![Value::Message(pose1), Value::Message(pose2)];

    let mut message = MessageValue::new();
    message.insert("header".into(), Value::Message(header));
    message.insert("poses".into(), Value::Array(poses));

    assert_eq!(
        format!("{}", Value::Message(message)),
        r#"
header: 
  frame_id: "abc"
  seq: 22
  stamp: 123.1
poses: 
  - 
    orientation: 
      w: 7
      x: 4
      y: 5
      z: 6
    position: 
      x: 1
      y: 2
      z: 3
  - 
    orientation: 
      w: 14
      x: 11
      y: 12
      z: 13
    position: 
      x: 8
      y: 9
      z: 10"#
    )
}
