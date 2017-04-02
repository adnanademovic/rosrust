use std::collections::{self, LinkedList, HashMap, HashSet};
use super::msg::Msg;
use super::error::{Result, ResultExt};
use std::fs::File;
use std::path::Path;

pub fn calculate_md5(message_map: &HashMap<(String, String), Msg>)
                     -> Result<HashMap<(String, String), String>> {
    let mut result = HashMap::<(String, String), String>::new();
    while result.len() < message_map.len() {
        let mut changed = false;
        for (key, value) in message_map {
            if result.contains_key(key) {
                continue;
            }
            if let Ok(answer) = value.calculate_md5(&result) {
                result.insert(key.clone(), answer);
                changed = true;
            }
        }
        if !changed {
            break;
        }
    }
    if result.len() < message_map.len() {
        bail!("Message map does not contain all needed elements");
    }
    Ok(result)
}

pub fn generate_message_definition(message_map: &HashMap<(String, String), Msg>,
                                   message: &Msg)
                                   -> Result<String> {
    let mut handled_messages = HashSet::<(String, String)>::new();
    let mut result = message.source.clone();
    let mut pending = message.dependencies().into_iter().collect::<LinkedList<_>>();
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

pub fn get_message_map(folders: &[&str],
                       messages: &[(&str, &str)])
                       -> Result<HashMap<(String, String), Msg>> {
    let mut result = HashMap::<(String, String), Msg>::new();
    let mut pending = messages.iter()
        .map(|&(key, val)| (key.into(), val.into()))
        .collect::<Vec<(String, String)>>();
    while let Some(value) = pending.pop() {
        let package = value.0.clone();
        let name = value.1.clone();
        if let collections::hash_map::Entry::Vacant(entry) = result.entry(value) {
            let message = get_message(folders, &package, &name)?;
            for dependency in &message.dependencies() {
                pending.push(dependency.clone());
            }
            entry.insert(message);
        }
    }
    Ok(result)
}

fn get_message(folders: &[&str], package: &str, name: &str) -> Result<Msg> {
    use std::io::Read;
    for folder in folders {
        let full_path =
            Path::new(&folder).join(&package).join("msg").join(&name).with_extension("msg");
        if let Ok(mut f) = File::open(&full_path) {
            let mut contents = String::new();
            f.read_to_string(&mut contents).chain_err(|| "Failed to read file to string!")?;
            return Msg::new(&package, &name, &contents);
        }
    }
    bail!(format!("Could not find requested message in provided folders: {}/{}",
                  package,
                  name));
}

#[cfg(test)]
mod tests {
    use super::*;

    lazy_static! {
        static ref FILEPATH: String = Path::new(file!())
            .parent().unwrap()
            .join("msg_examples")
            .to_str().unwrap()
            .into();
    }

    #[test]
    fn get_message_map_fetches_leaf_message() {
        let message_map = get_message_map(&[&FILEPATH], &[("geometry_msgs", "Point")]).unwrap();
        assert_eq!(message_map.len(), 1);
        assert!(message_map.contains_key(&("geometry_msgs".into(), "Point".into())));
    }

    #[test]
    fn get_message_map_fetches_message_and_dependencies() {
        let message_map = get_message_map(&[&FILEPATH], &[("geometry_msgs", "Pose")]).unwrap();
        assert_eq!(message_map.len(), 3);
        assert!(message_map.contains_key(&("geometry_msgs".into(), "Point".into())));
        assert!(message_map.contains_key(&("geometry_msgs".into(), "Pose".into())));
        assert!(message_map.contains_key(&("geometry_msgs".into(), "Quaternion".into())));
    }

    #[test]
    fn get_message_map_traverses_whole_dependency_tree() {
        let message_map = get_message_map(&[&FILEPATH], &[("geometry_msgs", "PoseStamped")])
            .unwrap();
        assert_eq!(message_map.len(), 5);
        assert!(message_map.contains_key(&("geometry_msgs".into(), "Point".into())));
        assert!(message_map.contains_key(&("geometry_msgs".into(), "Pose".into())));
        assert!(message_map.contains_key(&("geometry_msgs".into(), "PoseStamped".into())));
        assert!(message_map.contains_key(&("geometry_msgs".into(), "Quaternion".into())));
        assert!(message_map.contains_key(&("std_msgs".into(), "Header".into())));
    }

    #[test]
    fn get_message_map_traverses_all_passed_messages_dependency_tree() {
        let message_map = get_message_map(&[&FILEPATH],
                                          &[("geometry_msgs", "PoseStamped"),
                                            ("sensor_msgs", "Imu")])
            .unwrap();
        assert_eq!(message_map.len(), 7);
        assert!(message_map.contains_key(&("geometry_msgs".into(), "Vector3".into())));
        assert!(message_map.contains_key(&("geometry_msgs".into(), "Point".into())));
        assert!(message_map.contains_key(&("geometry_msgs".into(), "Pose".into())));
        assert!(message_map.contains_key(&("geometry_msgs".into(), "PoseStamped".into())));
        assert!(message_map.contains_key(&("geometry_msgs".into(), "Quaternion".into())));
        assert!(message_map.contains_key(&("sensor_msgs".into(), "Imu".into())));
        assert!(message_map.contains_key(&("std_msgs".into(), "Header".into())));
    }

    #[test]
    fn calculate_md5_works() {
        let message_map = get_message_map(&[&FILEPATH],
                                          &[("geometry_msgs", "PoseStamped"),
                                            ("sensor_msgs", "Imu")])
            .unwrap();
        let hashes = calculate_md5(&message_map).unwrap();
        assert_eq!(hashes.len(), 7);
        assert_eq!(*hashes.get(&("geometry_msgs".into(), "Vector3".into())).unwrap(),
                   "4a842b65f413084dc2b10fb484ea7f17".to_owned());
        assert_eq!(*hashes.get(&("geometry_msgs".into(), "Point".into())).unwrap(),
                   "4a842b65f413084dc2b10fb484ea7f17".to_owned());
        assert_eq!(*hashes.get(&("geometry_msgs".into(), "Quaternion".into())).unwrap(),
                   "a779879fadf0160734f906b8c19c7004".to_owned());
        assert_eq!(*hashes.get(&("geometry_msgs".into(), "Pose".into())).unwrap(),
                   "e45d45a5a1ce597b249e23fb30fc871f".to_owned());
        assert_eq!(*hashes.get(&("std_msgs".into(), "Header".into())).unwrap(),
                   "2176decaecbce78abc3b96ef049fabed".to_owned());
        assert_eq!(*hashes.get(&("geometry_msgs".into(), "PoseStamped".into())).unwrap(),
                   "d3812c3cbc69362b77dc0b19b345f8f5".to_owned());
        assert_eq!(*hashes.get(&("sensor_msgs".into(), "Imu".into())).unwrap(),
                   "6a62c6daae103f4ff57a132d6f95cec2".to_owned());
    }

    #[test]
    fn generate_message_definition_works() {
        let message_map = get_message_map(&[&FILEPATH], &[("geometry_msgs", "Vector3")]).unwrap();
        let definition = generate_message_definition(&message_map,
                                                     &message_map.get(&("geometry_msgs".into(),
                                                                "Vector3".into()))
                                                         .unwrap())
            .unwrap();
        assert_eq!(definition,
                   "# This represents a vector in free space. \n# It is only meant to represent \
                    a direction. Therefore, it does not\n# make sense to apply a translation to \
                    it (e.g., when applying a \n# generic rigid transformation to a Vector3, tf2 \
                    will only apply the\n# rotation). If you want your data to be translatable \
                    too, use the\n# geometry_msgs/Point message instead.\n\nfloat64 x\nfloat64 \
                    y\nfloat64 z\n");
        let message_map = get_message_map(&[&FILEPATH], &[("geometry_msgs", "PoseStamped")])
            .unwrap();
        let definition = generate_message_definition(&message_map,
                                                     &message_map.get(&("geometry_msgs".into(),
                                                                "PoseStamped".into()))
                                                         .unwrap())
            .unwrap();
        assert_eq!(definition,
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
");
    }
}
