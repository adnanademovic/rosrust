use crate::genmsg;
use proc_macro::TokenStream;
use quote::quote;
use std::env;
use std::ffi::OsStr;
use std::fs::read_dir;
use std::path::Path;

pub fn depend_on_messages(messages: &[&str], internal: bool) -> TokenStream {
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
    let paths_owned = cmake_paths
        .iter()
        .chain(cmake_alt_paths.iter())
        .chain(extra_paths.iter())
        .flat_map(|v| find_all_package_groups(&Path::new(v)))
        .collect::<Vec<String>>();
    let paths = paths_owned
        .iter()
        .map(String::as_str)
        .collect::<Vec<&str>>();
    let output = genmsg::depend_on_messages(paths.as_slice(), messages)
        .unwrap_or_else(|r| panic!("{}", r))
        .token_stream(&if internal {
            quote! { crate:: }
        } else {
            quote! { rosrust:: }
        });
    (quote! {#output}).into()
}

fn find_all_package_groups(root: &Path) -> Vec<String> {
    categorize_tree_folders(root)
        .into_iter()
        .filter(|v| v.has_msg_or_srv_grandchild)
        .map(|v| v.name)
        .collect()
}

fn categorize_tree_folders(root: &Path) -> Vec<FolderInfo> {
    if !root.is_dir() {
        return vec![];
    }
    let mut folders = vec![];
    let is_msg_or_srv =
        root.file_name() == Some(OsStr::new("msg")) || root.file_name() == Some(OsStr::new("srv"));
    let mut has_msg_or_srv_child = false;
    let mut has_msg_or_srv_grandchild = false;
    if let Ok(children) = read_dir(root) {
        for child in children.filter_map(Result::ok) {
            for folder in categorize_tree_folders(&child.path()) {
                has_msg_or_srv_child = has_msg_or_srv_child || folder.is_msg_or_srv;
                has_msg_or_srv_grandchild =
                    has_msg_or_srv_grandchild || folder.has_msg_or_srv_child;
                folders.push(folder);
            }
        }
    }
    folders.push(FolderInfo {
        name: root.to_str().unwrap_or("").into(),
        is_msg_or_srv,
        has_msg_or_srv_child,
        has_msg_or_srv_grandchild,
    });
    folders
}

struct FolderInfo {
    name: String,
    is_msg_or_srv: bool,
    has_msg_or_srv_child: bool,
    has_msg_or_srv_grandchild: bool,
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
