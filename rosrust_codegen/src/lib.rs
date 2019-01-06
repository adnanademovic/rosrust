#![deny(warnings)]
#![recursion_limit = "1024"]

extern crate proc_macro;
extern crate proc_macro2;
#[macro_use]
extern crate quote;
extern crate digest;
extern crate hex;
extern crate md5;
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

#[proc_macro]
pub fn rosmsg_include(input: TokenStream) -> TokenStream {
    let mut messages = Vec::new();
    let mut next_item = String::new();
    for item in input {
        match item.to_string().as_str() {
            "," => {
                messages.push(next_item);
                next_item = String::new();
            }
            s => next_item += s,
        }
    }
    let is_internal = next_item == "INTERNAL";
    if !is_internal && next_item != "" {
        messages.push(next_item);
    }
    let message_refs = messages.iter().map(|v| v.as_str()).collect::<Vec<&str>>();
    rosmsg_include::depend_on_messages(&message_refs, is_internal)
}
