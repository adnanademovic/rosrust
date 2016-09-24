pub trait RosMessage {
    fn msg_definition() -> String;
    fn md5sum() -> String;
    fn msg_type() -> String;
}
