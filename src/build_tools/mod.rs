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
    let variables = ["CMAKE_PREFIX_PATH", "ROSRUST_MSG_PATH"]
        .iter()
        .filter_map(|v| env::var(v).ok())
        .collect::<Vec<String>>();
    let paths = variables
        .iter()
        .flat_map(|v| v.split(":"))
        .collect::<Vec<&str>>();
    let output = genmsg::depend_on_messages(paths.as_slice(), messages).unwrap();
    let out_dir = env::var("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("msg.rs");
    let mut f = File::create(&dest_path).unwrap();
    write!(f, "{}", output).unwrap();
}
