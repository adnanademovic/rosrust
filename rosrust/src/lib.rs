#![deny(warnings)]
#![recursion_limit = "1024"]

extern crate byteorder;
extern crate ctrlc;
#[macro_use]
extern crate error_chain;
extern crate futures;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate log;
extern crate nix;
#[macro_use]
extern crate rosrust_codegen;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate crossbeam;
extern crate xml_rpc;
extern crate yaml_rust;

pub use api::raii::Service;
pub use api::{error, Clock, Parameter};
pub use rosmsg::RosMsg;
#[doc(hidden)]
pub use rosrust_codegen::*;
pub use singleton::*;
pub use tcpros::{Client, Message, PublisherStream, ServicePair};
pub use time::{Duration, Time};

pub mod api;
mod log_macros;
pub mod msg;
pub mod rosmsg;
mod rosxmlrpc;
pub mod singleton;
mod tcpros;
mod time;
