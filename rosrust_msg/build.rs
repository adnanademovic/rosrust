use std::ffi::OsStr;
use std::path::Path;
use std::process::Command;
use std::{env, fs};

fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();

    rerun_if_env_changed("OUT_DIR");
    rerun_if_env_changed("CMAKE_PREFIX_PATH");
    rerun_if_env_changed("ROSRUST_MSG_PATH");
    rerun_if_env_changed("ROSRUST_MSG_TYPES");

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
        .chain(extra_paths.iter());
    for path in paths {
        rerun_if_folder_content_changed(Path::new(path));
    }

    let messages;
    if let Ok(message_types_override) = env::var("ROSRUST_MSG_TYPES") {
        messages = message_types_override;
    } else {
        let rosmsg_list_output = Command::new("rosmsg").arg("list").output().unwrap().stdout;
        messages = std::str::from_utf8(&rosmsg_list_output)
            .unwrap()
            .lines()
            .collect::<Vec<&str>>()
            .join(",");
    }

    let file_name = format!("{}/{}", out_dir, "messages.rs");
    let file_content = format!("use rosrust;rosrust::rosmsg_include!({});", messages,);

    fs::write(&file_name, &file_content).unwrap();
}

fn rerun_if_file_changed(key: &str) {
    println!("cargo:rerun-if-changed={}", key);
}

fn rerun_if_env_changed(key: &str) {
    println!("cargo:rerun-if-env-changed={}", key);
}

pub fn rerun_if_folder_content_changed(folder: &Path) {
    if !folder.is_dir() {
        if folder.extension() == Some(OsStr::new("msg"))
            || folder.extension() == Some(OsStr::new("srv"))
        {
            if let Some(name) = folder.to_str() {
                rerun_if_file_changed(name);
            }
        }
        return;
    }
    if let Some(name) = folder.to_str() {
        rerun_if_file_changed(name);
    }
    if let Ok(children) = fs::read_dir(folder) {
        for child in children.filter_map(Result::ok) {
            rerun_if_folder_content_changed(&child.path());
        }
    }
}

fn append_share_folder(path: &str) -> Option<String> {
    Path::new(path).join("share").to_str().map(String::from)
}

fn append_src_folder(path: &str) -> Option<String> {
    Path::new(path)
        .join("..")
        .join("src")
        .to_str()
        .map(String::from)
}
