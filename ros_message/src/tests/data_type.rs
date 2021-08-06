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
    assert_eq!(
        DataType::I64.md5_string("", &hashes).unwrap(),
        "int64".to_owned()
    );
    assert_eq!(
        DataType::F32.md5_string("", &hashes).unwrap(),
        "float32".to_owned()
    );
    assert_eq!(
        DataType::String.md5_string("", &hashes).unwrap(),
        "string".to_owned()
    );
    assert_eq!(
        DataType::LocalStruct("xx".into())
            .md5_string("p1", &hashes)
            .unwrap(),
        "ABCD".to_owned()
    );
    assert_eq!(
        DataType::LocalStruct("xx".into())
            .md5_string("p2", &hashes)
            .unwrap(),
        "EFGH".to_owned()
    );
    assert_eq!(
        DataType::RemoteStruct(MessagePath::new("p1", "xx").expect("Unexpected bad message path"))
            .md5_string("p2", &hashes)
            .unwrap(),
        "ABCD".to_owned()
    );
}
