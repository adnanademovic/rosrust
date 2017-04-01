use std::collections::HashSet;
use super::helpers;
use super::error::Result;

pub fn depend_on_messages(folders: &[&str], messages: &[&str]) -> Result<String> {
    let mut output = Vec::<String>::new();
    output.push("pub mod msg {".into());
    let mut message_pairs = Vec::<(&str, &str)>::new();
    for message in messages {
        message_pairs.push(string_into_pair(message)?);
    }
    let message_map = helpers::get_message_map(folders, &message_pairs)?;
    let hashes = helpers::calculate_md5(&message_map)?;
    let packages = message_map.iter()
        .map(|(&(ref pack, ref _name), ref _value)| pack.clone())
        .collect::<HashSet<String>>();
    for package in packages {
        output.push(format!("    pub mod {} {{", package));
        let names = message_map.iter()
            .filter(|&(&(ref pack, ref _name), ref _value)| pack == &package)
            .map(|(&(ref _pack, ref name), ref _value)| name.clone())
            .collect::<HashSet<String>>();
        for name in names {
            let key = (package.clone(), name);
            let message = message_map.get(&key)
                .expect("Internal implementation contains mismatch in map keys");
            let hash = hashes.get(&key)
                .expect("Internal implementation contains mismatch in map keys");
            let definition = helpers::generate_message_definition(&message_map, &message)?;
            output.push(message.struct_string());
            output.push(format!("        impl ::rosrust::Message for {} {{", message.name));
            output.push(create_function("msg_definition", &definition));
            output.push(create_function("md5sum", &hash));
            output.push(create_function("msg_type", &message.get_type()));
            output.push("        }".into());
        }
        output.push("    }".into());
    }
    output.push("}".into());
    Ok(output.join("\n"))
}

fn create_function(name: &str, value: &str) -> String {
    format!(r#"
            fn {}() -> String {{
                {:?}.into()
            }}"#,
            name,
            value)
}

fn string_into_pair<'a>(input: &'a str) -> Result<(&'a str, &'a str)> {
    let mut parts = input.splitn(2, '/');
    let package = match parts.next() {
        Some(v) => v,
        None => bail!("Package string constains no parts: {}", input),
    };
    let name = match parts.next() {
        Some(v) => v,
        None => {
            bail!("Package string needs to be in package/name format: {}",
                  input)
        }
    };
    Ok((package, name))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    lazy_static! {
        static ref FILEPATH: String = Path::new(file!())
            .parent().unwrap()
            .join("msg_examples")
            .to_str().unwrap()
            .into();
    }

    #[test]
    fn depend_on_messages_printout() {
        let _data = depend_on_messages(&[&FILEPATH],
                                       &["geometry_msgs/PoseStamped", "sensor_msgs/Imu"])
            .unwrap();
        // TODO: actually test this output data
    }
}
