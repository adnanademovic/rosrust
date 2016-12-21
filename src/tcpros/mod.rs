pub use self::publisher::Publisher;
pub use self::subscriber::Subscriber;
pub use self::error::Error;

mod decoder;
mod encoder;
mod header;
mod error;
mod publisher;
mod subscriber;
mod streamfork;

pub trait Message {
    fn msg_definition() -> String;
    fn md5sum() -> String;
    fn msg_type() -> String;
}
