#![recursion_limit = "1024"]

extern crate byteorder;
extern crate crypto;
#[macro_use]
extern crate error_chain;
extern crate futures;
extern crate itertools;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate log;
extern crate nix;
extern crate regex;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_rosmsg;
extern crate xml_rpc;
extern crate yaml_rust;

pub use api::Ros;
pub use tcpros::{Client, PublisherStream, Message, ServicePair as Service};
pub use api::error;

mod api;
#[macro_use]
pub mod build_tools;
mod rosxmlrpc;
mod tcpros;
pub mod msg;
