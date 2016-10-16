extern crate byteorder;
extern crate hyper;
#[macro_use]
extern crate lazy_static;
extern crate libc;
extern crate regex;
extern crate rustc_serialize;
extern crate xml;

pub use api::Ros;

mod api;
pub mod build_tools;
mod rosxmlrpc;
pub mod tcpros;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {}
}
