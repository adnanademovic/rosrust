use genmsg;
use proc_macro::TokenStream;
use quote;
use std::env;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use syn;

fn read_rosmsg_includes_attribute(attrs: &Vec<syn::Attribute>) -> String {
    for attr in attrs {
        if attr.is_sugared_doc {
            continue;
        }
        if let Some(syn::Meta::NameValue(data)) = attr.interpret_meta() {
            if format!("{}", data.ident) != "rosmsg_includes" {
                continue;
            }
            match data.lit {
                syn::Lit::Str(msgs) => return msgs.value(),
                _ => panic!("rosmsg_includes attribute needs to be a string"),
            }
        }
    }
    panic!("rosmsg_includes attribute is not provided");
}

pub fn implement(ast: &syn::DeriveInput) -> TokenStream {
    let message_string = read_rosmsg_includes_attribute(&ast.attrs);
    let messages = message_string
        .split(',')
        .map(|v| v.trim())
        .collect::<Vec<&str>>();
    depend_on_messages(&messages, "::rosrust::");
    let output = quote!{};
    output.into()
}

fn depend_on_messages(messages: &[&str], crate_prefix: &str) {
    let cmake_paths = env::var("CMAKE_PREFIX_PATH")
        .unwrap_or_default()
        .split(':')
        .filter_map(append_share_folder)
        .collect::<Vec<String>>();
    let cmake_alt_paths = env::var("CMAKE_PREFIX_PATH")
        .unwrap_or_default()
        .split(':')
        .filter_map(append_src_folder)
        .collect::<Vec<String>>();
    let extra_paths = env::var("ROSRUST_MSG_PATH")
        .unwrap_or_default()
        .split(':')
        .map(String::from)
        .collect::<Vec<String>>();
    let paths = cmake_paths
        .iter()
        .chain(cmake_alt_paths.iter())
        .chain(extra_paths.iter())
        .map(|v| v.as_str())
        .collect::<Vec<&str>>();
    let output = genmsg::depend_on_messages(paths.as_slice(), messages, crate_prefix).unwrap();
    panic!(output);
}

fn append_share_folder(path: &str) -> Option<String> {
    Path::new(path).join("share").to_str().map(|v| v.to_owned())
}

fn append_src_folder(path: &str) -> Option<String> {
    Path::new(path)
        .join("..")
        .join("src")
        .to_str()
        .map(|v| v.to_owned())
}
