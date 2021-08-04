use super::*;
use crate::{DataType, MessagePath};

#[test]
fn match_field_matches_legal_field() {
    assert_eq!(
        FieldLine {
            field_type: "geom_msgs/Twist".into(),
            field_name: "myname".into(),
        },
        match_field("geom_msgs/Twist   myname").unwrap()
    );
}

#[test]
fn match_vector_field_matches_legal_field() {
    assert_eq!(
        FieldLine {
            field_type: "geom_msgs/Twist".into(),
            field_name: "myname".into(),
        },
        match_vector_field("geom_msgs/Twist [  ]   myname").unwrap()
    );
}

#[test]
fn match_array_field_matches_legal_field() {
    assert_eq!(
        (
            FieldLine {
                field_type: "geom_msgs/Twist".into(),
                field_name: "myname".into(),
            },
            127,
        ),
        match_array_field("geom_msgs/Twist   [   127 ]   myname").unwrap()
    );
}

#[test]
fn match_const_string_matches_legal_field() {
    assert_eq!(
        (
            FieldLine {
                field_type: "string".into(),
                field_name: "myname".into(),
            },
            "this is # data".into(),
        ),
        match_const_string("string   myname  =  this is # data").unwrap()
    );
}

#[test]
fn match_const_numeric_matches_legal_field() {
    assert_eq!(
        (
            FieldLine {
                field_type: "mytype".into(),
                field_name: "myname".into(),
            },
            "-444".into(),
        ),
        match_const_numeric("mytype   myname  =  -444").unwrap()
    );
}

#[test]
fn match_line_works_on_legal_data() {
    assert!(match_line("#just a comment").is_none());
    assert!(match_line("#  YOLO !   ").is_none());
    assert!(match_line("      ").is_none());

    assert_eq!(
        FieldInfo {
            datatype: DataType::RemoteStruct(
                MessagePath::new("geom_msgs", "Twist").expect("Unexpected bad message path")
            ),
            name: "myname".into(),
            case: FieldCase::Unit,
        },
        match_line("  geom_msgs/Twist   myname    # this clearly should succeed")
            .unwrap()
            .unwrap()
    );

    assert_eq!(
        FieldInfo {
            datatype: DataType::RemoteStruct(
                MessagePath::new("geom_msgs", "Twist").expect("Unexpected bad message path")
            ),
            name: "myname".into(),
            case: FieldCase::Vector,
        },
        match_line("  geom_msgs/Twist [  ]   myname  # ...")
            .unwrap()
            .unwrap()
    );

    assert_eq!(
        FieldInfo {
            datatype: DataType::U8(false),
            name: "myname".into(),
            case: FieldCase::Array(127),
        },
        match_line("  char   [   127 ]   myname# comment")
            .unwrap()
            .unwrap()
    );
    assert_eq!(
        FieldInfo {
            datatype: DataType::String,
            name: "myname".into(),
            case: FieldCase::Const("this is # data".into()),
        },
        match_line("  string  myname =   this is # data  ")
            .unwrap()
            .unwrap()
    );
    assert_eq!(
        FieldInfo {
            datatype: DataType::RemoteStruct(
                MessagePath::new("geom_msgs", "Twist").expect("Unexpected bad message path")
            ),
            name: "myname".into(),
            case: FieldCase::Const("-444".into()),
        },
        match_line("  geom_msgs/Twist  myname =   -444 # data  ")
            .unwrap()
            .unwrap()
    );
}

#[test]
fn match_lines_parses_real_messages() {
    let data = match_lines(include_str!(
        "../../../msg_examples/geometry_msgs/msg/TwistWithCovariance.msg"
    ))
    .unwrap();
    assert_eq!(
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
        ],
        data
    );

    let data = match_lines(include_str!(
        "../../../msg_examples/geometry_msgs/msg/PoseStamped.msg"
    ))
    .unwrap();
    assert_eq!(
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
        ],
        data
    );
}
