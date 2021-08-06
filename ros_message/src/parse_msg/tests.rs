use super::*;

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
        FieldInfo::new("geom_msgs/Twist", "myname", FieldCase::Unit).unwrap(),
        match_line("  geom_msgs/Twist   myname    # this clearly should succeed")
            .unwrap()
            .unwrap()
    );

    assert_eq!(
        FieldInfo::new("geom_msgs/Twist", "myname", FieldCase::Vector).unwrap(),
        match_line("  geom_msgs/Twist [  ]   myname  # ...")
            .unwrap()
            .unwrap()
    );

    assert_eq!(
        FieldInfo::new("char", "myname", FieldCase::Array(127)).unwrap(),
        match_line("  char   [   127 ]   myname# comment")
            .unwrap()
            .unwrap()
    );
    assert_eq!(
        FieldInfo::new(
            "string",
            "myname",
            FieldCase::Const("this is # data".into()),
        )
        .unwrap(),
        match_line("  string  myname =   this is # data  ")
            .unwrap()
            .unwrap()
    );
    assert_eq!(
        FieldInfo::new("geom_msgs/Twist", "myname", FieldCase::Const("-444".into())).unwrap(),
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
            FieldInfo::new("Twist", "twist", FieldCase::Unit).unwrap(),
            FieldInfo::new("float64", "covariance", FieldCase::Array(36)).unwrap(),
        ],
        data
    );

    let data = match_lines(include_str!(
        "../../../msg_examples/geometry_msgs/msg/PoseStamped.msg"
    ))
    .unwrap();
    assert_eq!(
        vec![
            FieldInfo::new("std_msgs/Header", "header", FieldCase::Unit).unwrap(),
            FieldInfo::new("Pose", "pose", FieldCase::Unit).unwrap(),
        ],
        data
    );
}
