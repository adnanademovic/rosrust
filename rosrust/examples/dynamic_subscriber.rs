use env_logger;
use rosrust;
use rosrust::{DynamicMsg, MsgValue};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

mod msg {
    rosrust::rosmsg_include!(std_msgs / String);
}

fn main() {
    env_logger::init();

    // Initialize node
    rosrust::init("listener");

    let dynamic_msg: Arc<Mutex<HashMap<String, DynamicMsg>>> = Arc::default();

    // Create subscriber
    // The subscriber is stopped when the returned object is destroyed
    let subscriber_info = rosrust::subscribe_with_ids_and_headers(
        "chatter",
        2,
        {
            let dynamic_msg = Arc::clone(&dynamic_msg);
            move |v: rosrust::RawMessage, id: &str| {
                rosrust::ros_info!("Received from '{}'", id);
                let current_msg = dynamic_msg.lock().unwrap();
                if let Some(ref current_msg) = current_msg.get(id) {
                    if let Ok(data) = current_msg.decode(std::io::Cursor::new(v.0)) {
                        // Callback for handling received messages
                        rosrust::ros_info!("Decoded as: {}", MsgValue::Message(data));
                    }
                }
            }
        },
        {
            let dynamic_msg = Arc::clone(&dynamic_msg);
            move |mut headers: HashMap<String, String>| {
                let caller_id = match headers.remove("callerid") {
                    Some(v) => v,
                    None => return,
                };
                let mut current_msg = dynamic_msg.lock().unwrap();
                rosrust::ros_info!("Connected to publisher with headers: {:#?}", headers);
                match DynamicMsg::from_headers(headers).ok() {
                    Some(v) => {
                        current_msg.insert(caller_id, v);
                    }
                    None => {
                        current_msg.remove(&caller_id);
                    }
                }
            }
        },
    )
    .unwrap();

    let log_names = rosrust::param("~log_names").unwrap().get().unwrap_or(false);

    if log_names {
        let rate = rosrust::rate(1.0);
        while rosrust::is_ok() {
            rosrust::ros_info!("Publisher uris: {:?}", subscriber_info.publisher_uris());
            rate.sleep();
        }
    } else {
        // Block the thread until a shutdown signal is received
        rosrust::spin();
    }
}
