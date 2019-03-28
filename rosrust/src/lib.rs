#![deny(warnings)]
#![recursion_limit = "1024"]

#[macro_use]
extern crate error_chain;

pub use crate::api::raii::{Publisher, Service, Subscriber};
pub use crate::api::{error, Clock, Parameter};
#[doc(hidden)]
pub use crate::rosmsg::RosMsg;
pub use crate::singleton::*;
pub use crate::tcpros::{Client, ClientResponse, Message, ServicePair};
pub use crate::time::{Duration, Time};
#[doc(hidden)]
pub use rosrust_codegen::*;

pub mod api;
mod log_macros;
#[doc(hidden)]
pub mod msg;
#[doc(hidden)]
pub mod rosmsg;
mod rosxmlrpc;
pub mod singleton;
mod tcpros;
mod time;
mod util;
