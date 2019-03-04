use crate::error::Result;
use crate::helpers;
use crate::output_layout;
use std::collections::HashSet;

pub fn depend_on_messages(folders: &[&str], messages: &[&str]) -> Result<output_layout::Layout> {
    let mut output = output_layout::Layout {
        packages: Vec::new(),
    };
    let mut message_pairs = Vec::<(&str, &str)>::new();
    for message in messages {
        message_pairs.push(string_into_pair(message)?);
    }
    let message_map = helpers::get_message_map(folders, &message_pairs)?;
    let hashes = helpers::calculate_md5(&message_map)?;
    let packages = message_map
        .messages
        .iter()
        .map(|(&(ref pack, ref _name), _value)| pack.clone())
        .chain(
            message_map
                .services
                .iter()
                .map(|&(ref pack, ref _name)| pack.clone()),
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
            .filter(|&(&(ref pack, ref _name), _value)| pack == &package)
            .map(|(&(ref _pack, ref name), _value)| name.clone())
            .collect::<HashSet<String>>();
        for name in &names {
            let key = (package.clone(), name.clone());
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
            });
        }
        let names = message_map
            .services
            .iter()
            .filter(|&&(ref pack, ref _name)| pack == &package)
            .map(|&(ref _pack, ref name)| name.clone())
            .collect::<HashSet<String>>();
        for name in &names {
            let md5sum = hashes
                .get(&(package.clone(), name.clone()))
                .expect("Internal implementation contains mismatch in map keys")
                .clone();
            let msg_type = format!("{}/{}", package, name);
            package_data.services.push(output_layout::Service {
                name: name.clone(),
                md5sum,
                msg_type,
            })
        }
        output.packages.push(package_data);
    }
    Ok(output)
}

fn string_into_pair(input: &str) -> Result<(&str, &str)> {
    let mut parts = input.splitn(2, '/');
    let package = match parts.next() {
        Some(v) => v,
        None => bail!("Package string constains no parts: {}", input),
    };
    let name = match parts.next() {
        Some(v) => v,
        None => bail!(
            "Package string needs to be in package/name format: {}",
            input
        ),
    };
    Ok((package, name))
}
