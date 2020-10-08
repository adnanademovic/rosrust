use crate::error::{ErrorKind, Result, ResultExt};
use crate::message_path::MessagePath;
use crate::msg::{Msg, Srv};
use error_chain::bail;
use lazy_static::lazy_static;
use regex::RegexBuilder;
use std;
use std::collections::{HashMap, HashSet, LinkedList};
use std::fs::{read_dir, File};
use std::path::{Path, PathBuf};

pub fn calculate_md5(message_map: &MessageMap) -> Result<HashMap<MessagePath, String>> {
    let mut representations = HashMap::<MessagePath, String>::new();
    let mut hashes = HashMap::<MessagePath, String>::new();
    while hashes.len() < message_map.messages.len() {
        let mut changed = false;
        for (key, value) in &message_map.messages {
            if hashes.contains_key(key) {
                continue;
            }
            if let Ok(answer) = value.get_md5_representation(&hashes) {
                hashes.insert(key.clone(), calculate_md5_from_representation(&answer));
                representations.insert(key.clone(), answer);
                changed = true;
            }
        }
        if !changed {
            break;
        }
    }
    for message in message_map.services.keys() {
        let key_req = MessagePath::new(&message.package, format!("{}Req", message.name));
        let key_res = MessagePath::new(&message.package, format!("{}Res", message.name));
        let req = match representations.get(&key_req) {
            Some(v) => v,
            None => bail!("Message map does not contain all needed elements"),
        };
        let res = match representations.get(&key_res) {
            Some(v) => v,
            None => bail!("Message map does not contain all needed elements"),
        };
        hashes.insert(
            message.clone(),
            calculate_md5_from_representation(&format!("{}{}", req, res)),
        );
    }
    if hashes.len() < message_map.messages.len() + message_map.services.len() {
        bail!("Message map does not contain all needed elements");
    }
    Ok(hashes)
}

fn calculate_md5_from_representation(v: &str) -> String {
    use md5::{Digest, Md5};
    let mut hasher = Md5::new();
    hasher.update(v);
    hex::encode(hasher.finalize())
}

pub fn generate_message_definition<S: std::hash::BuildHasher>(
    message_map: &HashMap<MessagePath, Msg, S>,
    message: &Msg,
) -> Result<String> {
    let mut handled_messages = HashSet::<MessagePath>::new();
    let mut result = message.source.clone();
    let mut pending = message
        .dependencies()
        .into_iter()
        .collect::<LinkedList<_>>();
    while let Some(value) = pending.pop_front() {
        if handled_messages.contains(&value) {
            continue;
        }
        handled_messages.insert(value.clone());
        result += "\n\n========================================";
        result += "========================================";
        result += &format!("\nMSG: {}\n", value);
        let message = match message_map.get(&value) {
            Some(msg) => msg,
            None => bail!("Message map does not contain all needed elements"),
        };
        for dependency in message.dependencies() {
            pending.push_back(dependency);
        }
        result += &message.source;
    }
    result += "\n";
    Ok(result)
}

pub struct MessageMap {
    pub messages: HashMap<MessagePath, Msg>,
    pub services: HashMap<MessagePath, Srv>,
}

pub fn get_message_map(
    ignore_bad_messages: bool,
    folders: &[&str],
    message_paths: &[MessagePath],
) -> Result<MessageMap> {
    let mut message_locations = HashMap::new();
    let mut service_locations = HashMap::new();

    let mut messages_and_services = vec![];
    for folder in folders {
        messages_and_services.append(&mut find_all_messages_and_services(Path::new(folder)));
    }

    for (message_path, file_path, message_type) in messages_and_services {
        match message_type {
            MessageType::Message => message_locations.insert(message_path, file_path),
            MessageType::Service => service_locations.insert(message_path, file_path),
        };
    }

    let mut messages = HashMap::new();
    let mut services = HashMap::new();
    let mut pending = message_paths.to_vec();
    while let Some(message_path) = pending.pop() {
        if messages.contains_key(&message_path) {
            continue;
        }
        match get_message_or_service(
            ignore_bad_messages,
            folders,
            &message_locations,
            &service_locations,
            message_path,
        )? {
            MessageCase::Message(message) => {
                for dependency in &message.dependencies() {
                    pending.push(dependency.clone());
                }
                messages.insert(message.path.clone(), message);
            }
            MessageCase::Service(service, req, res) => {
                for dependency in &req.dependencies() {
                    pending.push(dependency.clone());
                }
                for dependency in &res.dependencies() {
                    pending.push(dependency.clone());
                }
                messages.insert(req.path.clone(), req);
                messages.insert(res.path.clone(), res);
                services.insert(service.path.clone(), service);
            }
        }
    }
    Ok(MessageMap { messages, services })
}

enum MessageType {
    Message,
    Service,
}

fn find_all_messages_and_services(root: &Path) -> Vec<(MessagePath, PathBuf, MessageType)> {
    if !root.is_dir() {
        return identify_message_or_service(root).into_iter().collect();
    }
    let mut items = vec![];
    if let Ok(children) = read_dir(root) {
        for child in children.filter_map(|v| v.ok()) {
            items.append(&mut find_all_messages_and_services(&child.path()));
        }
    }
    items
}

fn identify_message_or_service(filename: &Path) -> Option<(MessagePath, PathBuf, MessageType)> {
    let extension = filename.extension()?;
    let message = filename.file_stem()?;
    let parent = filename.parent()?;
    let grandparent = parent.parent()?;
    let package = grandparent.file_name()?;
    if Some(extension) != parent.file_name() {
        return None;
    }
    let message_type = match extension.to_str() {
        Some("msg") => MessageType::Message,
        Some("srv") => MessageType::Service,
        _ => return None,
    };
    Some((
        MessagePath::new(package.to_str()?, message.to_str()?),
        filename.into(),
        message_type,
    ))
}

enum MessageCase {
    Message(Msg),
    Service(Srv, Msg, Msg),
}

lazy_static! {
    static ref IN_MEMORY_MESSAGES: HashMap<MessagePath, &'static str> =
        generate_in_memory_messages();
}

fn generate_in_memory_messages() -> HashMap<MessagePath, &'static str> {
    let mut output = HashMap::new();
    output.insert(
        MessagePath::new("rosgraph_msgs", "Clock"),
        include_str!("msg_examples/rosgraph_msgs/msg/Clock.msg"),
    );
    output.insert(
        MessagePath::new("rosgraph_msgs", "Log"),
        include_str!("msg_examples/rosgraph_msgs/msg/Log.msg"),
    );
    output.insert(
        MessagePath::new("std_msgs", "Header"),
        include_str!("msg_examples/std_msgs/msg/Header.msg"),
    );
    output
}

#[allow(clippy::trivial_regex)]
fn get_message_or_service(
    ignore_bad_messages: bool,
    folders: &[&str],
    message_locations: &HashMap<MessagePath, PathBuf>,
    service_locations: &HashMap<MessagePath, PathBuf>,
    message: MessagePath,
) -> Result<MessageCase> {
    use std::io::Read;

    let package = &message.package;
    let name = &message.name;

    if let Some(full_path) = message_locations.get(&message) {
        if let Ok(mut f) = File::open(full_path) {
            let mut contents = String::new();
            f.read_to_string(&mut contents)
                .chain_err(|| "Failed to read file to string!")?;
            return create_message(message, &contents, ignore_bad_messages)
                .map(MessageCase::Message);
        }
    }
    if let Some(full_path) = service_locations.get(&message) {
        if let Ok(mut f) = File::open(full_path) {
            let mut contents = String::new();
            f.read_to_string(&mut contents)
                .chain_err(|| "Failed to read file to string!")?;
            let re = RegexBuilder::new("^---$").multi_line(true).build()?;
            let (req, res) = match re.split(&contents).collect::<Vec<_>>().as_slice() {
                &[req] => (req, ""),
                &[req, res] => (req, res),
                &[] => bail!("Service {} does not have any content", message),
                v => bail!("Service {} is split into {} parts", message, v.len()),
            };
            let req = create_message(
                MessagePath::new(package, format!("{}Req", name)),
                req,
                ignore_bad_messages,
            )?;
            let res = create_message(
                MessagePath::new(package, format!("{}Res", name)),
                res,
                ignore_bad_messages,
            )?;
            let service = Srv {
                path: message,
                source: contents,
            };
            return Ok(MessageCase::Service(service, req, res));
        }
    }
    if let Some(contents) = IN_MEMORY_MESSAGES.get(&message) {
        return Msg::new(message, contents).map(MessageCase::Message);
    }
    if ignore_bad_messages {
        return Msg::new(message, "").map(MessageCase::Message);
    }
    bail!(ErrorKind::MessageNotFound(
        message.to_string(),
        folders.join("\n"),
    ))
}

fn create_message(message: MessagePath, contents: &str, ignore_bad_messages: bool) -> Result<Msg> {
    Msg::new(message.clone(), contents).or_else(|err| {
        if ignore_bad_messages {
            Msg::new(message, "")
        } else {
            Err(err)
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    static FILEPATH: &'static str = "src/msg_examples";

    #[test]
    fn get_message_map_fetches_leaf_message() {
        let message_map = get_message_map(
            false,
            &[FILEPATH],
            &[MessagePath::new("geometry_msgs", "Point")],
        )
        .unwrap()
        .messages;
        assert_eq!(message_map.len(), 1);
        assert!(message_map.contains_key(&MessagePath::new("geometry_msgs", "Point")));
    }

    #[test]
    fn get_message_map_fetches_message_and_dependencies() {
        let message_map = get_message_map(
            false,
            &[FILEPATH],
            &[MessagePath::new("geometry_msgs", "Pose")],
        )
        .unwrap()
        .messages;
        assert_eq!(message_map.len(), 3);
        assert!(message_map.contains_key(&MessagePath::new("geometry_msgs", "Point")));
        assert!(message_map.contains_key(&MessagePath::new("geometry_msgs", "Pose")));
        assert!(message_map.contains_key(&MessagePath::new("geometry_msgs", "Quaternion")));
    }

    #[test]
    fn get_message_map_traverses_whole_dependency_tree() {
        let message_map = get_message_map(
            false,
            &[FILEPATH],
            &[MessagePath::new("geometry_msgs", "PoseStamped")],
        )
        .unwrap()
        .messages;
        assert_eq!(message_map.len(), 5);
        assert!(message_map.contains_key(&MessagePath::new("geometry_msgs", "Point")));
        assert!(message_map.contains_key(&MessagePath::new("geometry_msgs", "Pose")));
        assert!(message_map.contains_key(&MessagePath::new("geometry_msgs", "PoseStamped")));
        assert!(message_map.contains_key(&MessagePath::new("geometry_msgs", "Quaternion")));
        assert!(message_map.contains_key(&MessagePath::new("std_msgs", "Header")));
    }

    #[test]
    fn get_message_map_traverses_all_passed_messages_dependency_tree() {
        let message_map = get_message_map(
            false,
            &[FILEPATH],
            &[
                MessagePath::new("geometry_msgs", "PoseStamped"),
                MessagePath::new("sensor_msgs", "Imu"),
                MessagePath::new("rosgraph_msgs", "Clock"),
                MessagePath::new("rosgraph_msgs", "Log"),
            ],
        )
        .unwrap()
        .messages;
        assert_eq!(message_map.len(), 9);
        assert!(message_map.contains_key(&MessagePath::new("geometry_msgs", "Vector3")));
        assert!(message_map.contains_key(&MessagePath::new("geometry_msgs", "Point")));
        assert!(message_map.contains_key(&MessagePath::new("geometry_msgs", "Pose")));
        assert!(message_map.contains_key(&MessagePath::new("geometry_msgs", "PoseStamped")));
        assert!(message_map.contains_key(&MessagePath::new("geometry_msgs", "Quaternion")));
        assert!(message_map.contains_key(&MessagePath::new("sensor_msgs", "Imu")));
        assert!(message_map.contains_key(&MessagePath::new("std_msgs", "Header")));
        assert!(message_map.contains_key(&MessagePath::new("rosgraph_msgs", "Clock")));
        assert!(message_map.contains_key(&MessagePath::new("rosgraph_msgs", "Log")));
    }

    #[test]
    fn calculate_md5_works() {
        let message_map = get_message_map(
            false,
            &[FILEPATH],
            &[
                MessagePath::new("geometry_msgs", "PoseStamped"),
                MessagePath::new("sensor_msgs", "Imu"),
                MessagePath::new("rosgraph_msgs", "Clock"),
                MessagePath::new("rosgraph_msgs", "Log"),
            ],
        )
        .unwrap();
        let hashes = calculate_md5(&message_map).unwrap();
        assert_eq!(hashes.len(), 9);
        assert_eq!(
            *hashes
                .get(&MessagePath::new("geometry_msgs", "Vector3"))
                .unwrap(),
            "4a842b65f413084dc2b10fb484ea7f17".to_owned()
        );
        assert_eq!(
            *hashes
                .get(&MessagePath::new("geometry_msgs", "Point"))
                .unwrap(),
            "4a842b65f413084dc2b10fb484ea7f17".to_owned()
        );
        assert_eq!(
            *hashes
                .get(&MessagePath::new("geometry_msgs", "Quaternion"))
                .unwrap(),
            "a779879fadf0160734f906b8c19c7004".to_owned()
        );
        assert_eq!(
            *hashes
                .get(&MessagePath::new("geometry_msgs", "Pose"))
                .unwrap(),
            "e45d45a5a1ce597b249e23fb30fc871f".to_owned()
        );
        assert_eq!(
            *hashes.get(&MessagePath::new("std_msgs", "Header")).unwrap(),
            "2176decaecbce78abc3b96ef049fabed".to_owned()
        );
        assert_eq!(
            *hashes
                .get(&MessagePath::new("geometry_msgs", "PoseStamped"))
                .unwrap(),
            "d3812c3cbc69362b77dc0b19b345f8f5".to_owned()
        );
        assert_eq!(
            *hashes.get(&MessagePath::new("sensor_msgs", "Imu")).unwrap(),
            "6a62c6daae103f4ff57a132d6f95cec2".to_owned()
        );
        assert_eq!(
            *hashes
                .get(&MessagePath::new("rosgraph_msgs", "Clock"))
                .unwrap(),
            "a9c97c1d230cfc112e270351a944ee47".to_owned()
        );
        assert_eq!(
            *hashes
                .get(&MessagePath::new("rosgraph_msgs", "Log"))
                .unwrap(),
            "acffd30cd6b6de30f120938c17c593fb".to_owned()
        );
    }

    #[test]
    fn generate_message_definition_works() {
        let message_map = get_message_map(
            false,
            &[FILEPATH],
            &[MessagePath::new("geometry_msgs", "Vector3")],
        )
        .unwrap()
        .messages;
        let definition = generate_message_definition(
            &message_map,
            &message_map
                .get(&MessagePath::new("geometry_msgs", "Vector3"))
                .unwrap(),
        )
        .unwrap();
        assert_eq!(
            definition,
            "# This represents a vector in free space. \n# It is only meant to represent \
             a direction. Therefore, it does not\n# make sense to apply a translation to \
             it (e.g., when applying a \n# generic rigid transformation to a Vector3, tf2 \
             will only apply the\n# rotation). If you want your data to be translatable \
             too, use the\n# geometry_msgs/Point message instead.\n\nfloat64 x\nfloat64 \
             y\nfloat64 z\n"
        );
        let message_map = get_message_map(
            false,
            &[FILEPATH],
            &[MessagePath::new("geometry_msgs", "PoseStamped")],
        )
        .unwrap()
        .messages;
        let definition = generate_message_definition(
            &message_map,
            &message_map
                .get(&MessagePath::new("geometry_msgs", "PoseStamped"))
                .unwrap(),
        )
        .unwrap();
        assert_eq!(
            definition,
            "# A Pose with reference coordinate frame and timestamp\n\
Header header\n\
Pose pose\n\
\n\
================================================================================\n\
MSG: std_msgs/Header\n\
# Standard metadata for higher-level stamped data types.\n\
# This is generally used to communicate timestamped data \n\
# in a particular coordinate frame.\n\
# \n\
# sequence ID: consecutively increasing ID \n\
uint32 seq\n\
#Two-integer timestamp that is expressed as:\n\
# * stamp.sec: seconds (stamp_secs) since epoch (in Python the variable is called 'secs')\n\
# * stamp.nsec: nanoseconds since stamp_secs (in Python the variable is called 'nsecs')\n\
# time-handling sugar is provided by the client library\n\
time stamp\n\
#Frame this data is associated with\n\
# 0: no frame\n\
# 1: global frame\n\
string frame_id\n\
\n\
================================================================================\n\
MSG: geometry_msgs/Pose\n\
# A representation of pose in free space, composed of position and orientation. \n\
Point position\n\
Quaternion orientation\n\
\n\
================================================================================\n\
MSG: geometry_msgs/Point\n\
# This contains the position of a point in free space\n\
float64 x\n\
float64 y\n\
float64 z\n\
\n\
================================================================================\n\
MSG: geometry_msgs/Quaternion\n\
# This represents an orientation in free space in quaternion form.\n\
\n\
float64 x\n\
float64 y\n\
float64 z\n\
float64 w\n\
"
        );
    }

    #[test]
    fn calculate_md5_works_for_services() {
        let message_map = get_message_map(
            false,
            &[FILEPATH],
            &[
                MessagePath::new("diagnostic_msgs", "AddDiagnostics"),
                MessagePath::new("simple_srv", "Something"),
            ],
        )
        .unwrap();
        let hashes = calculate_md5(&message_map).unwrap();
        assert_eq!(hashes.len(), 11);
        assert_eq!(
            *hashes
                .get(&MessagePath::new("diagnostic_msgs", "AddDiagnostics"))
                .unwrap(),
            "e6ac9bbde83d0d3186523c3687aecaee".to_owned()
        );
        assert_eq!(
            *hashes
                .get(&MessagePath::new("simple_srv", "Something"))
                .unwrap(),
            "63715c08716373d8624430cde1434192".to_owned()
        );
    }

    #[test]
    fn parse_tricky_srv_files() {
        get_message_map(
            false,
            &[FILEPATH],
            &[
                MessagePath::new("empty_srv", "Empty"),
                MessagePath::new("empty_req_srv", "EmptyRequest"),
                MessagePath::new("tricky_comment_srv", "TrickyComment"),
            ],
        )
        .unwrap();
    }
}
