use crate::{DataType, FieldCase, FieldInfo, MessagePath, Msg, Result};
use std::collections::{HashMap, HashSet};
use std::convert::TryInto;

#[test]
fn md5_string_is_correct() {
    assert_eq!(
        Msg::new(
            "std_msgs/String"
                .try_into()
                .expect("Unexpectedly bad message body"),
            "string data"
        )
        .unwrap()
        .calculate_md5(&HashMap::new())
        .unwrap(),
        "992ce8a1687cec8c8bd883ec73ca41d1".to_owned()
    );
    assert_eq!(
        Msg::new(
            "geometry_msgs/Point"
                .try_into()
                .expect("Unexpectedly bad message body"),
            include_str!("../../../msg_examples/geometry_msgs/msg/Point.msg"),
        )
        .unwrap()
        .calculate_md5(&HashMap::new())
        .unwrap(),
        "4a842b65f413084dc2b10fb484ea7f17".to_owned()
    );
    assert_eq!(
        Msg::new(
            "geometry_msgs/Quaternion"
                .try_into()
                .expect("Unexpectedly bad message body"),
            include_str!("../../../msg_examples/geometry_msgs/msg/Quaternion.msg"),
        )
        .unwrap()
        .calculate_md5(&HashMap::new())
        .unwrap(),
        "a779879fadf0160734f906b8c19c7004".to_owned()
    );
    let mut hashes = HashMap::new();
    hashes.insert(
        "geometry_msgs/Point"
            .try_into()
            .expect("Unexpectedly bad message path"),
        "4a842b65f413084dc2b10fb484ea7f17".to_owned(),
    );
    hashes.insert(
        "geometry_msgs/Quaternion"
            .try_into()
            .expect("Unexpectedly bad message path"),
        "a779879fadf0160734f906b8c19c7004".to_owned(),
    );
    assert_eq!(
        Msg::new(
            "geometry_msgs/Pose"
                .try_into()
                .expect("Unexpectedly bad message body"),
            include_str!("../../../msg_examples/geometry_msgs/msg/Pose.msg"),
        )
        .unwrap()
        .calculate_md5(&hashes)
        .unwrap(),
        "e45d45a5a1ce597b249e23fb30fc871f".to_owned()
    );
    let mut hashes = HashMap::new();
    hashes.insert(
        "geometry_msgs/Point"
            .try_into()
            .expect("Unexpectedly bad message path"),
        "4a842b65f413084dc2b10fb484ea7f17".to_owned(),
    );
    hashes.insert(
        "std_msgs/ColorRGBA"
            .try_into()
            .expect("Unexpectedly bad message path"),
        "a29a96539573343b1310c73607334b00".to_owned(),
    );
    hashes.insert(
        "std_msgs/Header"
            .try_into()
            .expect("Unexpectedly bad message path"),
        "2176decaecbce78abc3b96ef049fabed".to_owned(),
    );
    assert_eq!(
        Msg::new(
            "visualization_msgs/ImageMarker"
                .try_into()
                .expect("Unexpectedly bad message path"),
            include_str!("../../../msg_examples/visualization_msgs/msg/ImageMarker.msg"),
        )
        .unwrap()
        .calculate_md5(&hashes)
        .unwrap(),
        "1de93c67ec8858b831025a08fbf1b35c".to_owned()
    );
}

fn get_dependency_set(message: &Msg) -> Result<HashSet<MessagePath>> {
    Ok(message.dependencies()?.into_iter().collect())
}

#[test]
fn constructor_parses_real_message() {
    let data = Msg::new(
        "geometry_msgs/TwistWithCovariance".try_into().unwrap(),
        include_str!("../../../msg_examples/geometry_msgs/msg/TwistWithCovariance.msg"),
    )
    .unwrap();
    assert_eq!(data.path.package(), "geometry_msgs");
    assert_eq!(data.path.name(), "TwistWithCovariance");
    assert_eq!(
        data.fields,
        vec![
            FieldInfo {
                datatype: DataType::LocalStruct("Twist".into()),
                name: "twist".into(),
                case: FieldCase::Unit,
            },
            FieldInfo {
                datatype: DataType::F64,
                name: "covariance".into(),
                case: FieldCase::Array(36),
            },
        ]
    );
    let dependencies = get_dependency_set(&data).expect("Failed to get dependency set");
    assert_eq!(dependencies.len(), 1);
    assert!(dependencies.contains(
        &MessagePath::new("geometry_msgs", "Twist").expect("Unexpected bad message path")
    ));

    let data = Msg::new(
        "geometry_msgs/PoseStamped".try_into().unwrap(),
        include_str!("../../../msg_examples/geometry_msgs/msg/PoseStamped.msg"),
    )
    .unwrap();
    assert_eq!(data.path.package(), "geometry_msgs");
    assert_eq!(data.path.name(), "PoseStamped");
    assert_eq!(
        data.fields,
        vec![
            FieldInfo {
                datatype: DataType::RemoteStruct(
                    MessagePath::new("std_msgs", "Header").expect("Unexpected bad message path")
                ),
                name: "header".into(),
                case: FieldCase::Unit,
            },
            FieldInfo {
                datatype: DataType::LocalStruct("Pose".into()),
                name: "pose".into(),
                case: FieldCase::Unit,
            },
        ]
    );
    let dependencies = get_dependency_set(&data).expect("Failed to get dependency set");
    assert_eq!(dependencies.len(), 2);
    assert!(dependencies.contains(
        &MessagePath::new("geometry_msgs", "Pose").expect("Unexpected bad message path")
    ));
    assert!(dependencies
        .contains(&MessagePath::new("std_msgs", "Header").expect("Unexpected bad message path")));

    let data = Msg::new(
        "sensor_msgs/Imu".try_into().unwrap(),
        include_str!("../../../msg_examples/sensor_msgs/msg/Imu.msg"),
    )
    .unwrap();
    assert_eq!(data.path.package(), "sensor_msgs");
    assert_eq!(data.path.name(), "Imu");
    assert_eq!(
        data.fields,
        vec![
            FieldInfo {
                datatype: DataType::RemoteStruct(
                    MessagePath::new("std_msgs", "Header").expect("Unexpected bad message path")
                ),
                name: "header".into(),
                case: FieldCase::Unit,
            },
            FieldInfo {
                datatype: DataType::RemoteStruct(
                    MessagePath::new("geometry_msgs", "Quaternion")
                        .expect("Unexpected bad message path")
                ),
                name: "orientation".into(),
                case: FieldCase::Unit,
            },
            FieldInfo {
                datatype: DataType::F64,
                name: "orientation_covariance".into(),
                case: FieldCase::Array(9),
            },
            FieldInfo {
                datatype: DataType::RemoteStruct(
                    MessagePath::new("geometry_msgs", "Vector3")
                        .expect("Unexpected bad message path")
                ),
                name: "angular_velocity".into(),
                case: FieldCase::Unit,
            },
            FieldInfo {
                datatype: DataType::F64,
                name: "angular_velocity_covariance".into(),
                case: FieldCase::Array(9),
            },
            FieldInfo {
                datatype: DataType::RemoteStruct(
                    MessagePath::new("geometry_msgs", "Vector3")
                        .expect("Unexpected bad message path")
                ),
                name: "linear_acceleration".into(),
                case: FieldCase::Unit,
            },
            FieldInfo {
                datatype: DataType::F64,
                name: "linear_acceleration_covariance".into(),
                case: FieldCase::Array(9),
            },
        ]
    );
    let dependencies = get_dependency_set(&data).expect("Failed to get dependency set");
    assert_eq!(dependencies.len(), 3);
    assert!(dependencies.contains(
        &MessagePath::new("geometry_msgs", "Vector3").expect("Unexpected bad message path")
    ));
    assert!(dependencies.contains(
        &MessagePath::new("geometry_msgs", "Quaternion").expect("Unexpected bad message path")
    ));
    assert!(dependencies
        .contains(&MessagePath::new("std_msgs", "Header").expect("Unexpected bad message path")));
}
