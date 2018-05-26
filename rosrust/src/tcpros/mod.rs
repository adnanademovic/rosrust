pub use self::client::Client;
pub use self::error::Error;
pub use self::publisher::{Publisher, PublisherStream};
pub use self::service::Service;
pub use self::subscriber::Subscriber;
use serde::de::Deserialize;
use serde::ser::Serialize;

use Clock;
use std::sync::Arc;
use std::sync::atomic::AtomicUsize;

mod client;
pub mod error;
mod header;
mod publisher;
mod service;
mod subscriber;
mod util;

pub type ServiceResult<T> = Result<T, String>;

pub trait Message: Serialize + Deserialize<'static> + Send + 'static {
    fn msg_definition() -> String;
    fn md5sum() -> String;
    fn msg_type() -> String;
    fn set_header(&mut self, _clock: &Arc<Clock>, _seq: &Arc<AtomicUsize>) {}
}

pub trait ServicePair: Message {
    type Request: Serialize + Deserialize<'static> + Send + 'static;
    type Response: Serialize + Deserialize<'static> + Send + 'static;
}
