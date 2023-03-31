use crate::{FieldCase, FieldInfo, MessagePath, Msg, Value};
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

fn get_dependency_set(message: &Msg) -> HashSet<MessagePath> {
    message.dependencies().into_iter().collect()
}

#[test]
fn constructor_parses_real_message() {
    let data = Msg::new(
        "geometry_msgs/TwistWithCovariance".try_into().unwrap(),
        include_str!("../../../msg_examples/geometry_msgs/msg/TwistWithCovariance.msg"),
    )
    .unwrap();
    assert_eq!(data.path().package(), "geometry_msgs");
    assert_eq!(data.path().name(), "TwistWithCovariance");
    assert_eq!(
        data.fields(),
        vec![
            FieldInfo::new("Twist", "twist", FieldCase::Unit).unwrap(),
            FieldInfo::new("float64", "covariance", FieldCase::Array(36)).unwrap(),
        ]
    );
    let dependencies = get_dependency_set(&data);
    assert_eq!(dependencies.len(), 1);
    assert!(dependencies.contains(
        &MessagePath::new("geometry_msgs", "Twist").expect("Unexpected bad message path")
    ));

    let data = Msg::new(
        "geometry_msgs/PoseStamped".try_into().unwrap(),
        include_str!("../../../msg_examples/geometry_msgs/msg/PoseStamped.msg"),
    )
    .unwrap();
    assert_eq!(data.path().package(), "geometry_msgs");
    assert_eq!(data.path().name(), "PoseStamped");
    assert_eq!(
        data.fields(),
        vec![
            FieldInfo::new("std_msgs/Header", "header", FieldCase::Unit).unwrap(),
            FieldInfo::new("Pose", "pose", FieldCase::Unit).unwrap(),
        ]
    );
    let dependencies = get_dependency_set(&data);
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
    assert_eq!(data.path().package(), "sensor_msgs");
    assert_eq!(data.path().name(), "Imu");
    assert_eq!(
        data.fields(),
        vec![
            FieldInfo::new("std_msgs/Header", "header", FieldCase::Unit).unwrap(),
            FieldInfo::new("geometry_msgs/Quaternion", "orientation", FieldCase::Unit).unwrap(),
            FieldInfo::new("float64", "orientation_covariance", FieldCase::Array(9)).unwrap(),
            FieldInfo::new("geometry_msgs/Vector3", "angular_velocity", FieldCase::Unit).unwrap(),
            FieldInfo::new(
                "float64",
                "angular_velocity_covariance",
                FieldCase::Array(9),
            )
            .unwrap(),
            FieldInfo::new(
                "geometry_msgs/Vector3",
                "linear_acceleration",
                FieldCase::Unit,
            )
            .unwrap(),
            FieldInfo::new(
                "float64",
                "linear_acceleration_covariance",
                FieldCase::Array(9),
            )
            .unwrap(),
        ]
    );
    let dependencies = get_dependency_set(&data);
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

#[test]
fn has_header_checks_if_there_is_a_header_in_the_message_root() {
    let without_header = Msg::new(
        "geometry_msgs/TwistWithCovariance".try_into().unwrap(),
        include_str!("../../../msg_examples/geometry_msgs/msg/TwistWithCovariance.msg"),
    )
    .unwrap();
    assert!(!without_header.has_header());
    let with_header = Msg::new(
        "sensor_msgs/Imu".try_into().unwrap(),
        include_str!("../../../msg_examples/sensor_msgs/msg/Imu.msg"),
    )
    .unwrap();
    assert!(with_header.has_header());
}

#[test]
fn dependencies_lists_all_fields_the_message_depends_upon() {
    let msg = Msg::new(
        "geometry_msgs/PoseStamped".try_into().unwrap(),
        include_str!("../../../msg_examples/geometry_msgs/msg/PoseStamped.msg"),
    )
    .unwrap();
    assert_eq!(
        msg.dependencies(),
        vec![
            "std_msgs/Header".try_into().unwrap(),
            "geometry_msgs/Pose".try_into().unwrap(),
        ],
    );
}

#[test]
fn constants_returns_a_map_of_all_constants_in_message_root() {
    let msg = Msg::new(
        "benchmark_msgs/Overall".try_into().unwrap(),
        include_str!("../../../msg_examples/benchmark_msgs/msg/Overall.msg"),
    )
    .unwrap();
    let mut constants = HashMap::new();
    constants.insert("c_bool_t".into(), Value::Bool(true));
    constants.insert("c_bool_f".into(), Value::Bool(false));
    constants.insert("c_int8".into(), Value::I8(-55));
    constants.insert("c_int16".into(), Value::I16(-55));
    constants.insert("c_int32".into(), Value::I32(-55));
    constants.insert("c_int64".into(), Value::I64(-55));
    constants.insert("c_uint8".into(), Value::U8(55));
    constants.insert("c_uint16".into(), Value::U16(55));
    constants.insert("c_uint32".into(), Value::U32(55));
    constants.insert("c_uint64".into(), Value::U64(55));
    constants.insert("c_float32".into(), Value::F32(-55.0));
    constants.insert("c_float64".into(), Value::F64(-55.0));
    constants.insert(
        "c_string".into(),
        Value::String("Things 'in' here should \"be able\" # to go crazy with \\ escapes".into()),
    );
    assert_eq!(msg.constants(), constants);
}

#[test]
fn serialize_into_name_and_truncated_source_only() {
    assert_eq!(
        serde_json::to_value(
            Msg::new(
                "geometry_msgs/Quaternion"
                    .try_into()
                    .expect("Unexpectedly bad message body"),
                "# This represents an orientation in free space in quaternion form.\n\nfloat64 x\nfloat64 y\nfloat64 z\nfloat64 w\n",
            )
                .unwrap(),
        )
            .unwrap(),
        serde_json::from_str::<serde_json::Value>(
            r##"
            {
                "path": "geometry_msgs/Quaternion",
                "source": "# This represents an orientation in free space in quaternion form.\n\nfloat64 x\nfloat64 y\nfloat64 z\nfloat64 w"
            }
            "##,
        )
            .unwrap(),
    );
}

#[test]
fn deserialize_from_name_and_source() {
    assert_eq!(
        serde_json::from_str::<Msg>(
            r##"
            {
                "path": "geometry_msgs/Quaternion",
                "source": "# This represents an orientation in free space in quaternion form.\n\nfloat64 x\nfloat64 y\nfloat64 z\nfloat64 w\n\n\n\n"
            }
            "##,
        )
            .unwrap(),
        Msg::new(
            "geometry_msgs/Quaternion"
                .try_into()
                .expect("Unexpectedly bad message body"),
            "# This represents an orientation in free space in quaternion form.\n\nfloat64 x\nfloat64 y\nfloat64 z\nfloat64 w\n",
        )
            .unwrap(),
    );
}

#[test]
fn deserialize_rejects_bad_name_or_source() {
    assert!(
        serde_json::from_str::<Msg>(
            r##"
            {
                "path": "geometry_msgs/Quaternion",
                "source": "# This represents an orientation in free space in quaternion form.\n\nfloat64 x\nfloat64 y\nfloat64 z\nfloat64 w\n\n\n\n"
            }
            "##,
        )
            .is_ok(),
    );
    assert!(
        serde_json::from_str::<Msg>(
            r##"
            {
                "path": "Geometry_msgs/Quaternion",
                "source": "# This represents an orientation in free space in quaternion form.\n\nfloat64 x\nfloat64 y\nfloat64 z\nfloat64 w\n\n\n\n"
            }
            "##,
        )
            .is_err(),
    );
    assert!(
        serde_json::from_str::<Msg>(
            r##"
            {
                "path": "geometry_msgs/Quaternion",
                "source": "# This represents an orientation in free space in quaternion form.\n\nfloat64 _x\nfloat64 y\nfloat64 z\nfloat64 w\n\n\n\n"
            }
            "##,
        )
            .is_err(),
    );
}
