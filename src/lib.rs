#![recursion_limit = "1024"]

extern crate byteorder;
#[macro_use]
extern crate error_chain;
extern crate hyper;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate log;
extern crate nix;
extern crate regex;
pub extern crate rustc_serialize;
extern crate xml;

pub use api::Ros;
pub use tcpros::{Client, PublisherStream, Message, ServicePair as Service};
pub use api::error;
pub use rosxmlrpc::XmlRpcValue;

mod api;
#[macro_use]
pub mod build_tools;
mod rosxmlrpc;
mod tcpros;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {}
}
