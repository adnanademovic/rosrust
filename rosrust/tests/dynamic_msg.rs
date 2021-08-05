use ros_msg_parser::{MessageValue, Value};
use rosrust::{DynamicMsg, Message};

mod msg {
    rosrust::rosmsg_include!(geometry_msgs / PoseArray);
}

fn make_message() -> DynamicMsg {
    DynamicMsg::new(
        "geometry_msgs/PoseArray",
        &msg::geometry_msgs::PoseArray::msg_definition(),
    )
    .unwrap()
}

fn get_message_structure() -> MessageValue {
    let mut header = MessageValue::new();
    header.insert("seq".into(), Value::U32(22));
    header.insert("stamp".into(), Value::Time { sec: 123, nsec: 0 });
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

    message
}

fn get_message_bytes() -> Vec<u8> {
    vec![
        // header
        // header.seq
        22, 0, 0, 0, // 22
        // header.stamp
        // header.stamp sec
        123, 0, 0, 0, // header.stamp nsec
        0, 0, 0, 0, // header.frame_id
        // header.frame_id length
        3, 0, 0, 0, // 3
        // header.frame_id content
        97, 98, 99, // abc
        // poses
        // length
        2, 0, 0, 0, // 2
        // poses[0]
        // pose[0].position
        0, 0, 0, 0, 0, 0, 0xf0, 0x3f, // 1.0
        0, 0, 0, 0, 0, 0, 0x00, 0x40, // 2.0
        0, 0, 0, 0, 0, 0, 0x08, 0x40, // 3.0
        // pose[0].orientation
        0, 0, 0, 0, 0, 0, 0x10, 0x40, // 4.0
        0, 0, 0, 0, 0, 0, 0x14, 0x40, // 5.0
        0, 0, 0, 0, 0, 0, 0x18, 0x40, // 6.0
        0, 0, 0, 0, 0, 0, 0x1c, 0x40, // 7.0
        // poses[1]
        // pose[1].position
        0, 0, 0, 0, 0, 0, 0x20, 0x40, // 8.0
        0, 0, 0, 0, 0, 0, 0x22, 0x40, // 9.0
        0, 0, 0, 0, 0, 0, 0x24, 0x40, // 10.0
        // pose[1].orientation
        0, 0, 0, 0, 0, 0, 0x26, 0x40, // 11.0
        0, 0, 0, 0, 0, 0, 0x28, 0x40, // 12.0
        0, 0, 0, 0, 0, 0, 0x2a, 0x40, // 13.0
        0, 0, 0, 0, 0, 0, 0x2c, 0x40, // 14.0
    ]
}

#[test]
fn encodes_structures() {
    let dynamic_msg = make_message();
    let mut cursor = std::io::Cursor::new(vec![]);
    dynamic_msg
        .encode(&get_message_structure(), &mut cursor)
        .unwrap();
    let data = cursor.into_inner();
    assert_eq!(get_message_bytes(), data);
}

#[test]
fn decodes_structures() {
    let dynamic_msg = make_message();
    let cursor = std::io::Cursor::new(get_message_bytes());
    let data = dynamic_msg.decode(cursor).unwrap();
    assert_eq!(get_message_structure(), data);
}
