use crate::api::error::{self, ErrorKind, Result};
use crate::tcpros::{Subscriber, Topic};
use crate::util::FAILED_TO_LOCK;
use crate::Message;
use error_chain::bail;
use log::error;
use std::collections::{BTreeSet, HashMap};
use std::iter::FromIterator;
use std::sync::{Arc, Mutex};

#[derive(Clone, Default)]
pub struct SubscriptionsTracker {
    mapping: Arc<Mutex<HashMap<String, Subscriber>>>,
}

impl SubscriptionsTracker {
    pub fn add_publishers<T>(&self, topic: &str, name: &str, publishers: T) -> Result<()>
    where
        T: Iterator<Item = String>,
    {
        let mut last_error_message = None;
        if let Some(mut subscription) = self.mapping.lock().expect(FAILED_TO_LOCK).get_mut(topic) {
            let publisher_set: BTreeSet<String> = publishers.collect();
            subscription.limit_publishers_to(&publisher_set);
            for publisher in publisher_set {
                if let Err(err) = connect_to_publisher(&mut subscription, name, &publisher, topic) {
                    let info = err
                        .iter()
                        .map(|v| format!("{}", v))
                        .collect::<Vec<_>>()
                        .join("\nCaused by:");
                    error!("Failed to connect to publisher '{}': {}", publisher, info);
                    last_error_message = Some(err);
                }
            }
        }
        match last_error_message {
            None => Ok(()),
            Some(err) => Err(err),
        }
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

    pub fn add<T, F>(&self, name: &str, topic: &str, queue_size: usize, callback: F) -> Result<()>
    where
        T: Message,
        F: Fn(T, &str) + Send + 'static,
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
                let subscriber = Subscriber::new::<T, F>(name, topic, queue_size, callback);
                entry.insert(subscriber);
                Ok(())
            }
        }
    }

    #[inline]
    pub fn remove(&self, topic: &str) {
        self.mapping.lock().expect(FAILED_TO_LOCK).remove(topic);
    }

    #[inline]
    pub fn publisher_count(&self, topic: &str) -> usize {
        self.mapping
            .lock()
            .expect(FAILED_TO_LOCK)
            .get(topic)
            .map_or(0, Subscriber::publisher_count)
    }

    #[inline]
    pub fn publisher_uris(&self, topic: &str) -> Vec<String> {
        self.mapping
            .lock()
            .expect(FAILED_TO_LOCK)
            .get(topic)
            .map_or_else(Vec::new, Subscriber::publisher_uris)
    }
}

fn connect_to_publisher(
    subscriber: &mut Subscriber,
    caller_id: &str,
    publisher: &str,
    topic: &str,
) -> Result<()> {
    if subscriber.is_connected_to(publisher) {
        return Ok(());
    }
    let (protocol, hostname, port) = request_topic(publisher, caller_id, topic)?;
    if protocol != "TCPROS" {
        bail!(ErrorKind::CommunicationIssue(format!(
            "Publisher responded with a non-TCPROS protocol: {}",
            protocol
        )))
    }
    subscriber
        .connect_to(publisher, (hostname.as_str(), port as u16))
        .map_err(|err| ErrorKind::Io(err).into())
}

fn request_topic(
    publisher_uri: &str,
    caller_id: &str,
    topic: &str,
) -> error::rosxmlrpc::Result<(String, String, i32)> {
    use crate::rosxmlrpc::error::ResultExt;
    let (_code, _message, protocols): (i32, String, (String, String, i32)) = xml_rpc::Client::new()
        .map_err(error::rosxmlrpc::ErrorKind::ForeignXmlRpc)?
        .call(
            &publisher_uri
                .parse()
                .chain_err(|| error::rosxmlrpc::ErrorKind::BadUri(publisher_uri.into()))?,
            "requestTopic",
            &(caller_id, topic, [["TCPROS"]]),
        )
        .chain_err(|| error::rosxmlrpc::ErrorKind::TopicConnectionError(topic.to_owned()))?
        .map_err(|_| "error")?;
    Ok(protocols)
}
