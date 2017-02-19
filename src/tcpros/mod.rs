use rustc_serialize::{Decodable, Encodable};
use serde::ser::Serialize;
use serde::de::Deserialize;
pub use self::publisher::{Publisher, PublisherStream};
pub use self::subscriber::Subscriber;
pub use self::service::Service;
pub use self::client::Client;
pub use self::error::Error;

mod decoder;
mod header;
pub mod error;
mod publisher;
mod subscriber;
mod streamfork;
mod service;
mod client;

pub type ServiceResult<T> = Result<T, String>;

pub trait Message
    : Decodable + Encodable + Serialize + Deserialize + Send + 'static {
    fn msg_definition() -> String;
    fn md5sum() -> String;
    fn msg_type() -> String;
}

pub trait ServicePair: Message {
    type Request: Encodable + Decodable + Serialize + Deserialize + Send + 'static;
    type Response: Encodable + Decodable + Serialize + Deserialize + Send + 'static;
}
