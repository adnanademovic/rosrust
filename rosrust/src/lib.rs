#![deny(warnings)]
#![recursion_limit = "1024"]

#[macro_use]
extern crate error_chain;

pub use crate::api::raii::Service;
pub use crate::api::{error, Clock, Parameter};
pub use crate::rosmsg::RosMsg;
pub use crate::singleton::*;
pub use crate::tcpros::{Client, Message, PublisherStream, ServicePair};
pub use crate::time::{Duration, Time};
#[doc(hidden)]
pub use rosrust_codegen::*;

pub mod api;
mod log_macros;
pub mod msg;
pub mod rosmsg;
mod rosxmlrpc;
pub mod singleton;
mod tcpros;
mod time;
