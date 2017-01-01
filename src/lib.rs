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
extern crate rustc_serialize;
extern crate xml;

pub use api::Ros;
pub use rosxmlrpc::XmlRpcValue;
pub use tcpros::Message;
pub use tcpros::ServicePair as Service;

mod api;
pub mod build_tools;
mod rosxmlrpc;
mod tcpros;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {}
}
