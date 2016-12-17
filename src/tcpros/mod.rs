pub use self::encoder::Encoder;
pub use self::decoder::Decoder;

pub mod decoder;
pub mod encoder;
pub mod error;
pub mod header;
pub mod publisher;
pub mod subscriber;

pub trait Message {
    fn msg_definition() -> String;
    fn md5sum() -> String;
    fn msg_type() -> String;
}
