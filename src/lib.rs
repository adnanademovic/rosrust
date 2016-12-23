extern crate byteorder;
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

mod api;
pub mod build_tools;
pub mod rosxmlrpc;
pub mod tcpros;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {}
}
