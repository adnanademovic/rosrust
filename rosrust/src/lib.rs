#![recursion_limit = "1024"]

extern crate byteorder;
extern crate crypto;
#[macro_use]
extern crate error_chain;
extern crate futures;
extern crate itertools;
#[cfg(test)]
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate log;
extern crate nix;
extern crate regex;
#[macro_use]
extern crate rosrust_codegen;
extern crate serde;
extern crate serde_rosmsg;
extern crate xml_rpc;
extern crate yaml_rust;

pub use api::Ros;
pub use tcpros::{Client, PublisherStream, Message, ServicePair as Service};
pub use api::error;
pub use time::{Duration, Time};

mod api;
mod rosxmlrpc;
mod tcpros;
mod time;

rosmsg_include!();
