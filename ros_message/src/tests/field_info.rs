use crate::{FieldCase, FieldInfo, MessagePath, Value};
use std::collections::HashMap;

#[test]
fn md5_string_is_correct() {
    let mut hashes = HashMap::new();
    hashes.insert(
        MessagePath::new("p1", "xx").expect("Unexpected bad message path"),
        "ABCD".to_owned(),
    );
    hashes.insert(
        MessagePath::new("p2", "xx").expect("Unexpected bad message path"),
        "EFGH".to_owned(),
    );
    assert_eq!(
        FieldInfo::new("int64", "abc", FieldCase::Unit)
            .unwrap()
            .md5_string("", &hashes)
            .unwrap(),
        "int64 abc".to_owned()
    );
    assert_eq!(
        FieldInfo::new("float32", "abc", FieldCase::Array(3))
            .unwrap()
            .md5_string("", &hashes)
            .unwrap(),
        "float32[3] abc".to_owned()
    );
    assert_eq!(
        FieldInfo::new("int32", "abc", FieldCase::Vector)
            .unwrap()
            .md5_string("", &hashes)
            .unwrap(),
        "int32[] abc".to_owned()
    );
    assert_eq!(
        FieldInfo::new("string", "abc", FieldCase::Const("something".into()))
            .unwrap()
            .md5_string("", &hashes)
            .unwrap(),
        "string abc=something".to_owned()
    );
    assert_eq!(
        FieldInfo::new("xx", "abc", FieldCase::Vector)
            .unwrap()
            .md5_string("p1", &hashes)
            .unwrap(),
        "ABCD abc".to_owned()
    );
    assert_eq!(
        FieldInfo::new("xx", "abc", FieldCase::Array(3))
            .unwrap()
            .md5_string("p1", &hashes)
            .unwrap(),
        "ABCD abc".to_owned()
    );
    assert_eq!(
        FieldInfo::new("p2/xx", "abc", FieldCase::Unit)
            .unwrap()
            .md5_string("p1", &hashes)
            .unwrap(),
        "EFGH abc".to_owned()
    );
}

#[test]
fn display() {
    assert_eq!(
        format!(
            "{}",
            FieldInfo::new("int64", "abc", FieldCase::Unit).unwrap()
        ),
        "int64 abc".to_owned()
    );
    assert_eq!(
        format!(
            "{}",
            FieldInfo::new("float32", "abc", FieldCase::Array(3)).unwrap()
        ),
        "float32[3] abc".to_owned()
    );
    assert_eq!(
        format!(
            "{}",
            FieldInfo::new("int32", "abc", FieldCase::Vector).unwrap()
        ),
        "int32[] abc".to_owned()
    );
    assert_eq!(
        format!(
            "{}",
            FieldInfo::new("string", "abc", FieldCase::Const("something".into())).unwrap()
        ),
        "string abc=something".to_owned()
    );
    assert_eq!(
        format!(
            "{}",
            FieldInfo::new("xx", "abc", FieldCase::Vector).unwrap()
        ),
        "xx[] abc".to_owned()
    );
    assert_eq!(
        format!(
            "{}",
            FieldInfo::new("xx", "abc", FieldCase::Array(3)).unwrap()
        ),
        "xx[3] abc".to_owned()
    );
    assert_eq!(
        format!(
            "{}",
            FieldInfo::new("p2/xx", "abc", FieldCase::Unit).unwrap()
        ),
        "p2/xx abc".to_owned()
    );
}

#[test]
fn serialize_unit_field_case_as_string() {
    assert_eq!(
        serde_json::to_string(&FieldCase::Unit).unwrap(),
        r#""Unit""#,
    );
}

#[test]
fn serialize_vector_field_case_as_string() {
    assert_eq!(
        serde_json::to_string(&FieldCase::Vector).unwrap(),
        r#""Vector""#,
    );
}

#[test]
fn serialize_array_field_case_as_structure_with_length_integer() {
    assert_eq!(
        serde_json::to_string(&FieldCase::Array(12)).unwrap(),
        r#"{"Array":12}"#,
    );
}

#[test]
fn serialize_const_field_case_as_structure_with_value_string() {
    assert_eq!(
        serde_json::to_string(&FieldCase::Const("abc".into())).unwrap(),
        r#"{"Const":"abc"}"#,
    );
}

#[test]
fn deserialize_unit_field_case_from_string() {
    assert_eq!(
        serde_json::from_str::<FieldCase>(r#""Unit""#).unwrap(),
        FieldCase::Unit,
    );
}

#[test]
fn deserialize_vector_field_case_from_string() {
    assert_eq!(
        serde_json::from_str::<FieldCase>(r#""Vector""#).unwrap(),
        FieldCase::Vector,
    );
}

#[test]
fn deserialize_array_field_case_from_structure_with_length_integer() {
    assert_eq!(
        serde_json::from_str::<FieldCase>(r#"{"Array":12}"#).unwrap(),
        FieldCase::Array(12),
    );
}

#[test]
fn deserialize_const_field_case_from_structure_with_value_string() {
    assert_eq!(
        serde_json::from_str::<FieldCase>(r#"{"Const":"abc"}"#).unwrap(),
        FieldCase::Const("abc".into()),
    );
}

#[test]
fn serialize_field_info_as_correct_structure() {
    assert_eq!(
        serde_json::to_value(FieldInfo::new("p2/xx", "abc", FieldCase::Array(12)).unwrap())
            .unwrap(),
        serde_json::from_str::<serde_json::Value>(
            r#"
            {
                "datatype": "p2/xx",
                "name": "abc",
                "case": { "Array": 12 }
            }
            "#,
        )
        .unwrap(),
    );
    assert_eq!(
        serde_json::to_value(
            FieldInfo::new("int16", "abc", FieldCase::Const("33".into())).unwrap()
        )
        .unwrap(),
        serde_json::from_str::<serde_json::Value>(
            r#"
            {
                "datatype": "int16",
                "name": "abc",
                "case": { "Const": "33" }
            }
            "#,
        )
        .unwrap(),
    );
}

#[test]
fn deserialize_field_info_from_correct_structure() {
    assert_eq!(
        serde_json::from_str::<FieldInfo>(
            r#"
            {
                "datatype": "p2/xx",
                "name": "abc",
                "case": { "Array": 12 }
            }
            "#,
        )
        .unwrap(),
        FieldInfo::new("p2/xx", "abc", FieldCase::Array(12)).unwrap(),
    );
    assert_eq!(
        serde_json::from_str::<FieldInfo>(
            r#"
            {
                "datatype": "int16",
                "name": "abc",
                "case": { "Const": "33" }
            }
            "#,
        )
        .unwrap(),
        FieldInfo::new("int16", "abc", FieldCase::Const("33".into())).unwrap(),
    );
}

#[test]
fn deserialize_field_info_ensures_correct_const_value() {
    assert_eq!(
        serde_json::from_str::<FieldInfo>(
            r#"
            {
                "datatype": "int16",
                "name": "abc",
                "case": { "Const": "33" }
            }
            "#,
        )
        .unwrap()
        .const_value()
        .unwrap()
        .clone(),
        Value::I16(33),
    );
    assert!(serde_json::from_str::<FieldInfo>(
        r#"
            {
                "datatype": "int16",
                "name": "abc",
                "case": { "Const": "asdf" }
            }
            "#,
    )
    .is_err());
}
