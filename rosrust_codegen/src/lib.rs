#![recursion_limit = "1024"]

extern crate proc_macro;
extern crate proc_macro2;
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
mod output_layout;
mod rosmsg_include;

use proc_macro::TokenStream;

#[proc_macro_derive(RosmsgInclude, attributes(rosmsg_includes, rosrust_internal))]
pub fn rosmsg_include(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();
    rosmsg_include::implement(&ast)
}
