use api::error::{self, ErrorKind, Result};
use std::collections::HashMap;
use std::iter::FromIterator;
use std::sync::{Arc, Mutex};
use tcpros::{Subscriber, Topic};
use Message;

#[derive(Clone, Default)]
pub struct SubscriptionsTracker {
    mapping: Arc<Mutex<HashMap<String, Subscriber>>>,
}

impl SubscriptionsTracker {
    pub fn add_publishers<T>(&self, topic: &str, name: &str, publishers: T) -> Result<()>
    where
        T: Iterator<Item = String>,
    {
        if let Some(mut subscription) = self.mapping.lock().expect(FAILED_TO_LOCK).get_mut(topic) {
            for publisher in publishers {
                if let Err(err) = connect_to_publisher(&mut subscription, name, &publisher, topic) {
                    let info = err
                        .iter()
                        .map(|v| format!("{}", v))
                        .collect::<Vec<_>>()
                        .join("\nCaused by:");
                    error!("Failed to connect to publisher '{}': {}", publisher, info);
                    return Err(err);
                }
            }
        }
        Ok(())
    }

    #[inline]
    pub fn get_topics<T: FromIterator<Topic>>(&self) -> T {
        self.mapping
            .lock()
            .expect(FAILED_TO_LOCK)
            .values()
            .map(Subscriber::get_topic)
            .cloned()
            .collect()
    }

    pub fn add<T, F>(&self, name: &str, topic: &str, callback: F) -> Result<()>
    where
        T: Message,
        F: Fn(T) -> () + Send + 'static,
    {
        use std::collections::hash_map::Entry;
        match self
            .mapping
            .lock()
            .expect(FAILED_TO_LOCK)
            .entry(String::from(topic))
        {
            Entry::Occupied(..) => {
                error!("Duplicate subscription to topic '{}' attempted", topic);
                Err(ErrorKind::Duplicate("subscription".into()).into())
            }
            Entry::Vacant(entry) => {
                let subscriber = Subscriber::new::<T, F>(name, topic, callback);
                entry.insert(subscriber);
                Ok(())
            }
        }
    }

    #[inline]
    pub fn remove(&self, topic: &str) {
        self.mapping.lock().expect(FAILED_TO_LOCK).remove(topic);
    }
}

fn connect_to_publisher(
    subscriber: &mut Subscriber,
    caller_id: &str,
    publisher: &str,
    topic: &str,
) -> Result<()> {
    let (protocol, hostname, port) = request_topic(publisher, caller_id, topic)?;
    if protocol != "TCPROS" {
        bail!(
            "Publisher responded with a non-TCPROS protocol: {}",
            protocol
        )
    }
    subscriber
        .connect_to((hostname.as_str(), port as u16))
        .map_err(|err| ErrorKind::Io(err).into())
}

fn request_topic(
    publisher_uri: &str,
    caller_id: &str,
    topic: &str,
) -> error::rosxmlrpc::Result<(String, String, i32)> {
    let (_code, _message, protocols): (i32, String, (String, String, i32)) = xml_rpc::Client::new()
        .unwrap()
        .call(
            &publisher_uri.parse().unwrap(),
            "requestTopic",
            &(caller_id, topic, [["TCPROS"]]),
        ).unwrap()
        .unwrap();
    Ok(protocols)
}

static FAILED_TO_LOCK: &'static str = "Failed to acquire lock";
