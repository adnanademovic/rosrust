use std::collections::{HashMap, HashSet};
use std::{env, fs};

fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();

    let mut message_sets = HashMap::<&str, HashSet<&str>>::new();
    for (package, message) in rosrust_msg::MESSAGES {
        message_sets.entry(package).or_default().insert(message);
    }

    let mut actions = vec![];

    let action_parts = [
        "Action",
        "ActionFeedback",
        "ActionGoal",
        "ActionResult",
        "Feedback",
        "Goal",
        "Result",
    ];

    for (package, messages) in message_sets {
        for message in &messages {
            if !message.ends_with("Action") {
                continue;
            }
            let action = match message.rsplit_once("Action") {
                Some((v, _)) => v,
                None => continue,
            };

            let all_action_parts_present = action_parts
                .iter()
                .all(|suffix| messages.contains(format!("{}{}", action, suffix).as_str()));

            if !all_action_parts_present {
                continue;
            }
            actions.push((package, action));
        }
    }

    let action_lines = actions
        .iter()
        .map(|(package, action)| format!("crate::action!(rosrust_msg; {}: {});\n", package, action))
        .collect::<String>();

    let file_name = format!("{}/{}", out_dir, "actions.rs");

    fs::write(file_name, action_lines).unwrap();
}
