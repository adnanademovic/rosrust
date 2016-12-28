pub use self::publisher::{Publisher, PublisherStream};
pub use self::subscriber::Subscriber;
pub use self::service::Service;
pub use self::error::Error;

mod decoder;
mod encoder;
mod header;
mod error;
mod publisher;
mod subscriber;
mod streamfork;
mod service;

pub trait Message {
    fn msg_definition() -> String;
    fn md5sum() -> String;
    fn msg_type() -> String;
}
