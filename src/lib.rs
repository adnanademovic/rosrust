extern crate hyper;
extern crate libc;
extern crate rustc_serialize;
extern crate xml;

pub use api::Ros;

mod api;
mod rosxmlrpc;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {}
}
