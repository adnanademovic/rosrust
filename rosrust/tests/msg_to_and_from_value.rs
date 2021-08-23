use rosrust::{MsgMessage, MsgValue, Time};
use std::convert::TryInto;

mod msg {
    rosrust::rosmsg_include!(geometry_msgs / PoseArray);
}

fn make_typed_message() -> msg::geometry_msgs::PoseArray {
    msg::geometry_msgs::PoseArray {
        header: msg::std_msgs::Header {
            seq: 22,
            stamp: Time { sec: 123, nsec: 0 },
            frame_id: "abc".into(),
        },
        poses: vec![
            msg::geometry_msgs::Pose {
                position: msg::geometry_msgs::Point {
                    x: 1.0,
                    y: 2.0,
                    z: 3.0,
                },
                orientation: msg::geometry_msgs::Quaternion {
                    x: 4.0,
                    y: 5.0,
                    z: 6.0,
                    w: 7.0,
                },
            },
            msg::geometry_msgs::Pose {
                position: msg::geometry_msgs::Point {
                    x: 8.0,
                    y: 9.0,
                    z: 10.0,
                },
                orientation: msg::geometry_msgs::Quaternion {
                    x: 11.0,
                    y: 12.0,
                    z: 13.0,
                    w: 14.0,
                },
            },
        ],
    }
}

fn make_dynamic_message() -> MsgMessage {
    let mut header = MsgMessage::new();
    header.insert("seq".into(), MsgValue::U32(22));
    header.insert("stamp".into(), MsgValue::Time(Time { sec: 123, nsec: 0 }));
    header.insert("frame_id".into(), MsgValue::String("abc".into()));

    let mut position1 = MsgMessage::new();
    position1.insert("x".into(), MsgValue::F64(1.0));
    position1.insert("y".into(), MsgValue::F64(2.0));
    position1.insert("z".into(), MsgValue::F64(3.0));

    let mut orientation1 = MsgMessage::new();
    orientation1.insert("x".into(), MsgValue::F64(4.0));
    orientation1.insert("y".into(), MsgValue::F64(5.0));
    orientation1.insert("z".into(), MsgValue::F64(6.0));
    orientation1.insert("w".into(), MsgValue::F64(7.0));

    let mut pose1 = MsgMessage::new();
    pose1.insert("position".into(), MsgValue::Message(position1));
    pose1.insert("orientation".into(), MsgValue::Message(orientation1));

    let mut position2 = MsgMessage::new();
    position2.insert("x".into(), MsgValue::F64(8.0));
    position2.insert("y".into(), MsgValue::F64(9.0));
    position2.insert("z".into(), MsgValue::F64(10.0));

    let mut orientation2 = MsgMessage::new();
    orientation2.insert("x".into(), MsgValue::F64(11.0));
    orientation2.insert("y".into(), MsgValue::F64(12.0));
    orientation2.insert("z".into(), MsgValue::F64(13.0));
    orientation2.insert("w".into(), MsgValue::F64(14.0));

    let mut pose2 = MsgMessage::new();
    pose2.insert("position".into(), MsgValue::Message(position2));
    pose2.insert("orientation".into(), MsgValue::Message(orientation2));

    let poses = vec![MsgValue::Message(pose1), MsgValue::Message(pose2)];

    let mut message = MsgMessage::new();
    message.insert("header".into(), MsgValue::Message(header));
    message.insert("poses".into(), MsgValue::Array(poses));

    message
}

#[test]
fn typed_message_to_dynamic_value() {
    let typed_message = make_typed_message();
    let dynamic_value: MsgValue = typed_message.into();
    assert_eq!(dynamic_value, MsgValue::Message(make_dynamic_message()));
}

#[test]
fn typed_message_to_dynamic_message() {
    let typed_message = make_typed_message();
    let dynamic_message: MsgMessage = typed_message.into();
    assert_eq!(dynamic_message, make_dynamic_message());
}

#[test]
fn dynamic_value_to_typed_message() {
    let dynamic_value = MsgValue::Message(make_dynamic_message());
    let typed_message: msg::geometry_msgs::PoseArray = dynamic_value
        .try_into()
        .expect("Failed to perform conversion");
    assert_eq!(typed_message, make_typed_message());
}

#[test]
fn dynamic_message_to_typed_message() {
    let dynamic_message = make_dynamic_message();
    let typed_message: msg::geometry_msgs::PoseArray = dynamic_message
        .try_into()
        .expect("Failed to perform conversion");
    assert_eq!(typed_message, make_typed_message());
}
