use rustc_serialize::{Decodable, Encodable};
pub use self::publisher::{Publisher, PublisherStream};
pub use self::subscriber::Subscriber;
pub use self::service::Service;
pub use self::client::Client;
pub use self::error::Error;

mod decoder;
mod encoder;
mod header;
mod error;
mod publisher;
mod subscriber;
mod streamfork;
mod service;
mod client;

pub trait Message: Decodable + Encodable + Send + Sync + Clone + 'static {
    fn msg_definition() -> String;
    fn md5sum() -> String;
    fn msg_type() -> String;
}
