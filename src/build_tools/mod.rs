pub mod genmsg;
pub mod error;

use regex::Regex;
use std::collections::{HashMap, HashSet};
use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader, Read, Write};
use std::path::Path;

#[macro_export]
macro_rules! rosmsg_main {
    ($($msg:expr),*)=> {
        fn main() {
            $crate::build_tools::depend_on_messages(&[
            $(
                $msg,
            )*
            ]);
        }
    }
}

#[macro_export]
macro_rules! rosmsg_include {
    () => {include!(concat!(env!("OUT_DIR"), "/msg.rs"));}
}

pub fn depend_on_messages(messages: &[&str]) {
    let messages = messages.into_iter()
        .map(|v| {
            let parts = v.split("/").collect::<Vec<&str>>();
            if parts.len() != 2 {
                panic!("Message name \"{}\" should be in format package/name", v);
            }
            Message {
                package: parts.get(0).unwrap().to_owned().to_owned(),
                name: parts.get(1).unwrap().to_owned().to_owned(),
            }
        })
        .collect();
    let mut paths = env::var("CMAKE_PREFIX_PATH")
        .unwrap()
        .split(":")
        .map(|v| v.to_owned())
        .collect::<Vec<String>>();
    if let Ok(path) = env::var("ROSRUST_MSG_PATH") {
        paths.push(path);
    }
    let messages = get_message_map(&paths, &messages);
    let out_dir = env::var("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("msg.rs");
    let mut f = File::create(&dest_path).unwrap();
    f.write(b"extern crate rustc_serialize;\n").unwrap();
    f.write(b"mod msg {\n").unwrap();
    for (package, names) in messages {
        f.write_fmt(format_args!("pub mod {} {{\n", package)).unwrap();
        for name in names {
            f.write_fmt(format_args!("// Content for message: {}/{}\n", package, name)).unwrap();
            f.write_all(&append_message(&paths, &package, &name)).unwrap();
        }
        f.write(b"}\n").unwrap();
    }
    f.write(b"}\n").unwrap();
}

fn append_message(paths: &Vec<String>, package: &str, name: &str) -> Vec<u8> {
    for path in paths {
        let dest_path = Path::new(&path)
            .join("rust")
            .join(package)
            .join(name)
            .with_extension("rs");
        if let Ok(mut f) = File::open(&dest_path) {
            let mut data = vec![];
            f.read_to_end(&mut data).unwrap();
            return data;
        }
    }
    panic!("Missing message file for {}/{}", package, name);
}

#[derive(Clone)]
struct Message {
    pub package: String,
    pub name: String,
}

fn get_message_map(paths: &Vec<String>, msgs: &Vec<Message>) -> HashMap<String, HashSet<String>> {
    let mut result = HashMap::new();
    for message in get_messages(paths, msgs) {
        result.entry(message.package)
            .or_insert(HashSet::new())
            .insert(message.name);
    }
    result
}

fn get_messages(paths: &Vec<String>, msgs: &Vec<Message>) -> Vec<Message> {
    let child_dependencies = msgs.into_iter()
        .map(|v| get_dependencies(paths, v))
        .map(|v| get_messages(paths, &v));
    let mut dependencies = msgs.clone();
    for child_dep in child_dependencies {
        dependencies.extend_from_slice(&child_dep);
    }
    dependencies
}

fn get_dependencies(paths: &Vec<String>, msg: &Message) -> Vec<Message> {
    let mut dependencies = vec![];
    lazy_static! {
        static ref RE: Regex = Regex::new(r"^//\s+([^\s]+)\s+([^\s]+)$").unwrap();
    }
    for path in paths {
        let dest_path = Path::new(&path)
            .join("rust")
            .join(&msg.package)
            .join(&msg.name)
            .with_extension("rs");
        if let Ok(f) = File::open(&dest_path) {
            for line in BufReader::new(f).lines() {
                if let Some(capture) = RE.captures(&line.unwrap()) {
                    dependencies.push(Message {
                        package: capture.get(1).unwrap().as_str().to_owned(),
                        name: capture.get(2).unwrap().as_str().to_owned(),
                    });
                } else {
                    break;
                }
            }
            break;
        }
    }
    dependencies
}
