use crate::{DataType, MessagePath};
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
    assert_eq!(DataType::I64.md5_str("", &hashes).unwrap(), "int64");
    assert_eq!(DataType::F32.md5_str("", &hashes).unwrap(), "float32");
    assert_eq!(DataType::String.md5_str("", &hashes).unwrap(), "string");
    assert_eq!(
        DataType::LocalMessage("xx".into())
            .md5_str("p1", &hashes)
            .unwrap(),
        "ABCD",
    );
    assert_eq!(
        DataType::LocalMessage("xx".into())
            .md5_str("p2", &hashes)
            .unwrap(),
        "EFGH",
    );
    assert_eq!(
        DataType::GlobalMessage(MessagePath::new("p1", "xx").expect("Unexpected bad message path"))
            .md5_str("p2", &hashes)
            .unwrap(),
        "ABCD",
    );
}

#[test]
fn serialize_as_string() {
    assert_eq!(serde_json::to_string(&DataType::I64).unwrap(), "\"int64\"");
    assert_eq!(
        serde_json::to_string(&DataType::F32).unwrap(),
        "\"float32\"",
    );
    assert_eq!(
        serde_json::to_string(&DataType::String).unwrap(),
        "\"string\"",
    );
    assert_eq!(
        serde_json::to_string(&DataType::LocalMessage("xx".into())).unwrap(),
        "\"xx\"",
    );
    assert_eq!(
        serde_json::to_string(&DataType::GlobalMessage(
            MessagePath::new("p1", "xx").expect("Unexpected bad message path")
        ))
        .unwrap(),
        "\"p1/xx\"",
    );
}

#[test]
fn deserialize_from_string() {
    assert_eq!(
        serde_json::from_str::<DataType>("\"int64\"").unwrap(),
        DataType::I64,
    );
    assert_eq!(
        serde_json::from_str::<DataType>("\"float32\"").unwrap(),
        DataType::F32,
    );
    assert_eq!(
        serde_json::from_str::<DataType>("\"string\"").unwrap(),
        DataType::String,
    );
    assert_eq!(
        serde_json::from_str::<DataType>("\"xx\"").unwrap(),
        DataType::LocalMessage("xx".into()),
    );
    assert_eq!(
        serde_json::from_str::<DataType>("\"p1/xx\"").unwrap(),
        DataType::GlobalMessage(MessagePath::new("p1", "xx").expect("Unexpected bad message path")),
    );
}
