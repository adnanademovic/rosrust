use crate::MessagePath;

#[test]
fn package_names_must_be_at_least_two_characters() {
    MessagePath::new("foo", "MessageName").expect("Unexpected incorrect package name");
    MessagePath::new("fo", "MessageName").expect("Unexpected incorrect package name");
    MessagePath::new("f", "MessageName").expect_err("Unexpected correct package name");
}

#[test]
fn package_names_must_start_with_lowercase_alphabetic() {
    MessagePath::new("foo_123", "MessageName").expect("Unexpected incorrect package name");
    MessagePath::new("Foo_123", "MessageName").expect_err("Unexpected correct package name");
    MessagePath::new("1oo_123", "MessageName").expect_err("Unexpected correct package name");
    MessagePath::new("_oo_123", "MessageName").expect_err("Unexpected correct package name");
}

#[test]
fn package_names_must_not_contain_uppercase_anywhere() {
    MessagePath::new("foo_123", "MessageName").expect("Unexpected incorrect package name");
    MessagePath::new("foO_123", "MessageName").expect_err("Unexpected correct package name");
}

#[test]
fn package_names_are_limited_to_lowercase_alphanumeric_and_underscore() {
    MessagePath::new("foo_123", "MessageName").expect("Unexpected incorrect package name");
    MessagePath::new("foO_123", "MessageName").expect_err("Unexpected correct package name");
    MessagePath::new("foo-123", "MessageName").expect_err("Unexpected correct package name");
}

#[test]
fn package_names_must_not_contain_multiple_underscores_in_a_row() {
    MessagePath::new("foo_123_", "MessageName").expect("Unexpected incorrect package name");
    MessagePath::new("f_o_o_1_2_3_", "MessageName").expect("Unexpected incorrect package name");
    MessagePath::new("foo__123_", "MessageName").expect_err("Unexpected correct package name");
    MessagePath::new("foo___123_", "MessageName").expect_err("Unexpected correct package name");
}

#[test]
fn format_returns_the_full_message_name() {
    let path: MessagePath =
        MessagePath::new("foo_123", "MessageName").expect("Unexpected incorrect package name");
    assert_eq!(format!("{}", path), "foo_123/MessageName");
}

#[test]
fn serialize_as_string() {
    assert_eq!(
        serde_json::to_string(&MessagePath::new("foo_123", "MessageName").unwrap()).unwrap(),
        "\"foo_123/MessageName\"",
    );
}

#[test]
fn deserialize_from_string() {
    assert_eq!(
        serde_json::from_str::<MessagePath>("\"foo_123/MessageName\"").unwrap(),
        MessagePath::new("foo_123", "MessageName").unwrap(),
    );
}

#[test]
fn deserialize_checks_if_package_name_is_valid() {
    assert!(serde_json::from_str::<MessagePath>("\"foo_123/MessageName\"").is_ok());
    assert!(serde_json::from_str::<MessagePath>("\"Foo_123/MessageName\"").is_err());
}
