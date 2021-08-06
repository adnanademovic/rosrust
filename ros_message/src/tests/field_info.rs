use crate::{FieldCase, FieldInfo, MessagePath};
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
