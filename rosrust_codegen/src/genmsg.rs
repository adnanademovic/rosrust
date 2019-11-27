use crate::error::Result;
use crate::helpers;
use crate::helpers::MessageMap;
use crate::message_path::MessagePath;
use crate::output_layout;
use std::collections::{HashMap, HashSet};
use std::convert::TryInto;

pub fn depend_on_messages(
    ignore_bad_messages: bool,
    folders: &[&str],
    messages: &[&str],
) -> Result<output_layout::Layout> {
    let message_map = message_names_to_message_map(ignore_bad_messages, folders, messages)?;
    validate_message_paths(&message_map)?;
    message_map_to_layout(&message_map)
}

fn message_names_to_message_map(
    ignore_bad_messages: bool,
    folders: &[&str],
    messages: &[&str],
) -> Result<MessageMap> {
    let message_pairs = messages
        .iter()
        .copied()
        .map(TryInto::try_into)
        .collect::<Result<Vec<MessagePath>>>()?;
    helpers::get_message_map(ignore_bad_messages, folders, &message_pairs)
}

fn validate_message_paths(message_map: &MessageMap) -> Result<()> {
    for message in message_map.messages.keys() {
        message.validate()?;
    }
    for message in message_map.services.keys() {
        message.validate()?;
    }
    Ok(())
}

fn message_map_to_layout(message_map: &MessageMap) -> Result<output_layout::Layout> {
    let mut output = output_layout::Layout {
        packages: Vec::new(),
    };
    let hashes = helpers::calculate_md5(&message_map)?;
    let packages = message_map
        .messages
        .keys()
        .map(|message| message.package.clone())
        .chain(
            message_map
                .services
                .keys()
                .map(|message| message.package.clone()),
        )
        .collect::<HashSet<String>>();
    for package in packages {
        let mut package_data = output_layout::Package {
            name: package.clone(),
            messages: Vec::new(),
            services: Vec::new(),
        };
        let names = message_map
            .messages
            .iter()
            .filter(|&(message, _value)| &message.package == &package)
            .map(|(message, value)| (message.name.clone(), value.source.clone()))
            .collect::<HashMap<String, String>>();
        for (name, source) in names {
            let key = MessagePath::new(&package, name);
            let message = message_map
                .messages
                .get(&key)
                .expect("Internal implementation contains mismatch in map keys")
                .clone();
            let md5sum = hashes
                .get(&key)
                .expect("Internal implementation contains mismatch in map keys")
                .clone();
            let msg_definition =
                helpers::generate_message_definition(&message_map.messages, &message)?;
            let msg_type = message.get_type();
            package_data.messages.push(output_layout::Message {
                message,
                msg_definition,
                msg_type,
                md5sum,
                source,
            });
        }
        let names = message_map
            .services
            .iter()
            .filter(|&(message, _value)| &message.package == &package)
            .map(|(message, value)| (message.name.clone(), value.source.clone()))
            .collect::<HashMap<String, String>>();
        for (name, source) in names {
            let md5sum = hashes
                .get(&MessagePath::new(&package, &name))
                .expect("Internal implementation contains mismatch in map keys")
                .clone();
            let msg_type = format!("{}/{}", package, name);
            package_data.services.push(output_layout::Service {
                name,
                md5sum,
                msg_type,
                source,
            })
        }
        output.packages.push(package_data);
    }
    Ok(output)
}
