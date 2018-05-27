#![recursion_limit = "1024"]

extern crate proc_macro;
#[macro_use]
extern crate quote;
extern crate crypto;
extern crate syn;
#[macro_use]
extern crate error_chain;
#[macro_use]
extern crate lazy_static;
extern crate regex;

mod error;
mod genmsg;
mod helpers;
mod msg;
mod rosmsg_include;

/*
#[macro_export]
macro_rules! rosmsg_main {
    ($($msg:expr),*) => {
        fn main() {
            $crate::depend_on_messages(&[
            $(
                $msg,
            )*
            ], "::rosrust::");
        }
    }
}

#[macro_export]
macro_rules! rosmsg_main_no_prefix {
    ($($msg:expr),*) => {
        fn main() {
            $crate::depend_on_messages(&[
            $(
                $msg,
            )*
            ], "::");
        }
    }
}

#[macro_export]
macro_rules! rosmsg_include {
    ($msgs:expr) => {
        mod __rosrust_rosmsg_include {
            #[derive(RosmsgInclude)]
            #[rosmsg_includes=$msgs]
            struct _RosmsgIncludeDummy;
        }
        pub use self::__rosrust_rosmsg_include::*;
    }
}
*/

use proc_macro::TokenStream;

#[proc_macro_derive(RosmsgInclude, attributes(rosmsg_includes))]
pub fn rosmsg_include(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();
    rosmsg_include::implement(&ast)
}
