use std::collections::{self, HashMap};
use super::msg::Msg;
use super::error::{Result, ResultExt};
use std::fs::File;
use std::path::Path;

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
            for dependency in &message.dependencies {
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
        let full_path = Path::new(&folder).join(&package).join(&name).with_extension("msg");
        println!("PATH: {}", full_path.to_str().unwrap());
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
}
