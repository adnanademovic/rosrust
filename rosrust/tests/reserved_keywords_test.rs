// The message type visualization_msgs/ImageMarker has a field called "type"
// which is a reserved keyword in Rust.
//
// Compilation of this test makes sure this is handled with field renames.

mod msg {
    rosrust::rosmsg_include!(visualization_msgs / ImageMarker);
}

#[test]
fn field_names_are_correct() {
    let message = msg::visualization_msgs::ImageMarker::default();
    assert_eq!(message.id, 0);
    assert_eq!(message.type_, 0);
}
