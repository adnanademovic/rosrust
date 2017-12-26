#![recursion_limit = "1024"]

extern crate crypto;
#[macro_use]
extern crate error_chain;
#[macro_use]
extern crate lazy_static;
extern crate regex;

pub mod msg;
pub mod helpers;
pub mod error;
pub mod genmsg;

use std::env;
use std::fs::File;
use std::io::Write;
use std::path::Path;

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
    () => {include!(concat!(env!("OUT_DIR"), "/msg.rs"));}
}

pub fn depend_on_messages(messages: &[&str], crate_prefix: &str) {
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
    let out_dir = env::var("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("msg.rs");
    let mut f = File::create(&dest_path).unwrap();
    write!(f, "{}", output).unwrap();
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
