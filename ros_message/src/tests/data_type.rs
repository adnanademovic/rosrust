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
