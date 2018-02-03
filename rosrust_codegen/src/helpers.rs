use std::collections::{HashMap, HashSet, LinkedList};
use msg::Msg;
use error::{Result, ResultExt};
use std;
use std::fs::File;
use std::path::Path;
use regex::RegexBuilder;

pub fn calculate_md5(message_map: &MessageMap) -> Result<HashMap<(String, String), String>> {
    let mut representations = HashMap::<(String, String), String>::new();
    let mut hashes = HashMap::<(String, String), String>::new();
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
    for &(ref pack, ref name) in &message_map.services {
        let key_req = (pack.clone(), format!("{}Req", name));
        let key_res = (pack.clone(), format!("{}Res", name));
        let req = match representations.get(&key_req) {
            Some(v) => v,
            None => bail!("Message map does not contain all needed elements"),
        };
        let res = match representations.get(&key_res) {
            Some(v) => v,
            None => bail!("Message map does not contain all needed elements"),
        };
        hashes.insert(
            (pack.clone(), name.clone()),
            calculate_md5_from_representation(&format!("{}{}", req, res)),
        );
    }
    if hashes.len() < message_map.messages.len() + message_map.services.len() {
        bail!("Message map does not contain all needed elements");
    }
    Ok(hashes)
}

fn calculate_md5_from_representation(v: &str) -> String {
    use crypto::md5::Md5;
    use crypto::digest::Digest;
    let mut hasher = Md5::new();
    hasher.input_str(v);
    hasher.result_str()
}

pub fn generate_message_definition<S: std::hash::BuildHasher>(
    message_map: &HashMap<(String, String), Msg, S>,
    message: &Msg,
) -> Result<String> {
    let mut handled_messages = HashSet::<(String, String)>::new();
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
        result += &format!("\nMSG: {}/{}\n", value.0, value.1);
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
    pub messages: HashMap<(String, String), Msg>,
    pub services: HashSet<(String, String)>,
}

pub fn get_message_map(folders: &[&str], messages: &[(&str, &str)]) -> Result<MessageMap> {
    let mut msgs = HashMap::new();
    let mut srvs = HashSet::new();
    let mut pending = messages
        .iter()
        .map(|&(key, val)| (key.into(), val.into()))
        .collect::<Vec<(String, String)>>();
    while let Some(value) = pending.pop() {
        let package = value.0.clone();
        let name = value.1.clone();
        if !msgs.contains_key(&value) {
            match get_message(folders, &package, &name)? {
                MessageCase::Message(message) => {
                    for dependency in &message.dependencies() {
                        pending.push(dependency.clone());
                    }
                    msgs.insert(value, message);
                }
                MessageCase::Service(service_name, req, res) => {
                    for dependency in &req.dependencies() {
                        pending.push(dependency.clone());
                    }
                    for dependency in &res.dependencies() {
                        pending.push(dependency.clone());
                    }
                    msgs.insert((package.clone(), req.name.clone()), req);
                    msgs.insert((package.clone(), res.name.clone()), res);
                    srvs.insert((package, service_name));
                }
            };
        }
    }
    Ok(MessageMap {
        messages: msgs,
        services: srvs,
    })
}

enum MessageCase {
    Message(Msg),
    Service(String, Msg, Msg),
}

#[allow(unknown_lints, trivial_regex)]
fn get_message(folders: &[&str], package: &str, name: &str) -> Result<MessageCase> {
    use std::io::Read;
    for folder in folders {
        let full_path = Path::new(&folder)
            .join(&package)
            .join("msg")
            .join(&name)
            .with_extension("msg");
        if let Ok(mut f) = File::open(&full_path) {
            let mut contents = String::new();
            f.read_to_string(&mut contents)
                .chain_err(|| "Failed to read file to string!")?;
            return Msg::new(package, name, &contents).map(MessageCase::Message);
        }
        let full_path = Path::new(&folder)
            .join(&package)
            .join("srv")
            .join(&name)
            .with_extension("srv");
        if let Ok(mut f) = File::open(&full_path) {
            let mut contents = String::new();
            f.read_to_string(&mut contents)
                .chain_err(|| "Failed to read file to string!")?;
            let re = RegexBuilder::new("^---$").multi_line(true).build()?;
            let mut parts = re.split(&contents);
            let req = match parts.next() {
                Some(v) => v,
                None => bail!("Service needs to have content"),
            };
            let res = match parts.next() {
                Some(v) => v,
                None => "",
            };
            if parts.next().is_some() {
                bail!("Too many splits in service");
            }
            let req = Msg::new(package, &format!("{}Req", name), req)?;
            let res = Msg::new(package, &format!("{}Res", name), res)?;
            return Ok(MessageCase::Service(name.into(), req, res));
        }
    }
    bail!(format!(
        "Could not find requested message in provided folders: {}/{}",
        package, name
    ));
}

#[cfg(test)]
mod tests {
    use super::*;

    static FILEPATH: &'static str = "src/msg_examples";

    #[test]
    fn get_message_map_fetches_leaf_message() {
        let message_map = get_message_map(&[FILEPATH], &[("geometry_msgs", "Point")])
            .unwrap()
            .messages;
        assert_eq!(message_map.len(), 1);
        assert!(message_map.contains_key(&("geometry_msgs".into(), "Point".into()),));
    }

    #[test]
    fn get_message_map_fetches_message_and_dependencies() {
        let message_map = get_message_map(&[FILEPATH], &[("geometry_msgs", "Pose")])
            .unwrap()
            .messages;
        assert_eq!(message_map.len(), 3);
        assert!(message_map.contains_key(&("geometry_msgs".into(), "Point".into()),));
        assert!(message_map.contains_key(&("geometry_msgs".into(), "Pose".into()),));
        assert!(message_map.contains_key(&("geometry_msgs".into(), "Quaternion".into()),));
    }

    #[test]
    fn get_message_map_traverses_whole_dependency_tree() {
        let message_map = get_message_map(&[FILEPATH], &[("geometry_msgs", "PoseStamped")])
            .unwrap()
            .messages;
        assert_eq!(message_map.len(), 5);
        assert!(message_map.contains_key(&("geometry_msgs".into(), "Point".into()),));
        assert!(message_map.contains_key(&("geometry_msgs".into(), "Pose".into()),));
        assert!(message_map.contains_key(&("geometry_msgs".into(), "PoseStamped".into()),));
        assert!(message_map.contains_key(&("geometry_msgs".into(), "Quaternion".into()),));
        assert!(message_map.contains_key(&("std_msgs".into(), "Header".into()),));
    }

    #[test]
    fn get_message_map_traverses_all_passed_messages_dependency_tree() {
        let message_map = get_message_map(
            &[FILEPATH],
            &[
                ("geometry_msgs", "PoseStamped"),
                ("sensor_msgs", "Imu"),
                ("rosgraph_msgs", "Clock"),
                ("rosgraph_msgs", "Log"),
            ],
        ).unwrap()
            .messages;
        assert_eq!(message_map.len(), 9);
        assert!(message_map.contains_key(&("geometry_msgs".into(), "Vector3".into()),));
        assert!(message_map.contains_key(&("geometry_msgs".into(), "Point".into()),));
        assert!(message_map.contains_key(&("geometry_msgs".into(), "Pose".into()),));
        assert!(message_map.contains_key(&("geometry_msgs".into(), "PoseStamped".into()),));
        assert!(message_map.contains_key(&("geometry_msgs".into(), "Quaternion".into()),));
        assert!(message_map.contains_key(&("sensor_msgs".into(), "Imu".into()),));
        assert!(message_map.contains_key(&("std_msgs".into(), "Header".into()),));
        assert!(message_map.contains_key(&("rosgraph_msgs".into(), "Clock".into()),));
        assert!(message_map.contains_key(&("rosgraph_msgs".into(), "Log".into()),));
    }

    #[test]
    fn calculate_md5_works() {
        let message_map = get_message_map(
            &[FILEPATH],
            &[
                ("geometry_msgs", "PoseStamped"),
                ("sensor_msgs", "Imu"),
                ("rosgraph_msgs", "Clock"),
                ("rosgraph_msgs", "Log"),
            ],
        ).unwrap();
        let hashes = calculate_md5(&message_map).unwrap();
        assert_eq!(hashes.len(), 9);
        assert_eq!(
            *hashes
                .get(&("geometry_msgs".into(), "Vector3".into()))
                .unwrap(),
            "4a842b65f413084dc2b10fb484ea7f17".to_owned()
        );
        assert_eq!(
            *hashes
                .get(&("geometry_msgs".into(), "Point".into()))
                .unwrap(),
            "4a842b65f413084dc2b10fb484ea7f17".to_owned()
        );
        assert_eq!(
            *hashes
                .get(&("geometry_msgs".into(), "Quaternion".into()))
                .unwrap(),
            "a779879fadf0160734f906b8c19c7004".to_owned()
        );
        assert_eq!(
            *hashes
                .get(&("geometry_msgs".into(), "Pose".into()))
                .unwrap(),
            "e45d45a5a1ce597b249e23fb30fc871f".to_owned()
        );
        assert_eq!(
            *hashes.get(&("std_msgs".into(), "Header".into())).unwrap(),
            "2176decaecbce78abc3b96ef049fabed".to_owned()
        );
        assert_eq!(
            *hashes
                .get(&("geometry_msgs".into(), "PoseStamped".into()))
                .unwrap(),
            "d3812c3cbc69362b77dc0b19b345f8f5".to_owned()
        );
        assert_eq!(
            *hashes.get(&("sensor_msgs".into(), "Imu".into())).unwrap(),
            "6a62c6daae103f4ff57a132d6f95cec2".to_owned()
        );
        assert_eq!(
            *hashes
                .get(&("rosgraph_msgs".into(), "Clock".into()))
                .unwrap(),
            "a9c97c1d230cfc112e270351a944ee47".to_owned()
        );
        assert_eq!(
            *hashes.get(&("rosgraph_msgs".into(), "Log".into())).unwrap(),
            "acffd30cd6b6de30f120938c17c593fb".to_owned()
        );
    }

    #[test]
    fn generate_message_definition_works() {
        let message_map = get_message_map(&[FILEPATH], &[("geometry_msgs", "Vector3")])
            .unwrap()
            .messages;
        let definition = generate_message_definition(
            &message_map,
            &message_map
                .get(&("geometry_msgs".into(), "Vector3".into()))
                .unwrap(),
        ).unwrap();
        assert_eq!(
            definition,
            "# This represents a vector in free space. \n# It is only meant to represent \
             a direction. Therefore, it does not\n# make sense to apply a translation to \
             it (e.g., when applying a \n# generic rigid transformation to a Vector3, tf2 \
             will only apply the\n# rotation). If you want your data to be translatable \
             too, use the\n# geometry_msgs/Point message instead.\n\nfloat64 x\nfloat64 \
             y\nfloat64 z\n"
        );
        let message_map = get_message_map(&[FILEPATH], &[("geometry_msgs", "PoseStamped")])
            .unwrap()
            .messages;
        let definition = generate_message_definition(
            &message_map,
            &message_map
                .get(&("geometry_msgs".into(), "PoseStamped".into()))
                .unwrap(),
        ).unwrap();
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
            &[FILEPATH],
            &[
                ("diagnostic_msgs", "AddDiagnostics"),
                ("simple_srv", "Something"),
            ],
        ).unwrap();
        let hashes = calculate_md5(&message_map).unwrap();
        assert_eq!(hashes.len(), 11);
        assert_eq!(
            *hashes
                .get(&("diagnostic_msgs".into(), "AddDiagnostics".into()))
                .unwrap(),
            "e6ac9bbde83d0d3186523c3687aecaee".to_owned()
        );
        assert_eq!(
            *hashes
                .get(&("simple_srv".into(), "Something".into()))
                .unwrap(),
            "63715c08716373d8624430cde1434192".to_owned()
        );
    }

    #[test]
    fn parse_tricky_srv_files() {
        get_message_map(
            &[FILEPATH],
            &[
                ("empty_srv", "Empty"),
                ("empty_req_srv", "EmptyRequest"),
                ("tricky_comment_srv", "TrickyComment"),
            ],
        ).unwrap();
    }
}
