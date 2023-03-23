use crate::api::error::{self, ErrorKind, Result};
use crate::tcpros::{SubscriberRosConnection, Topic};
use crate::util::FAILED_TO_LOCK;
use crate::{Message, SubscriptionHandler};
use error_chain::bail;
use log::error;
use std::collections::{BTreeSet, HashMap};
use std::iter::FromIterator;
use std::sync::{Arc, Mutex};

#[derive(Clone, Default)]
pub struct SubscriptionsTracker {
    mapping: Arc<Mutex<HashMap<String, SubscriberRosConnection>>>,
}

impl SubscriptionsTracker {
    pub fn add_publishers<T>(&self, topic: &str, name: &str, publishers: T) -> Result<()>
    where
        T: Iterator<Item = String>,
    {
        let mut last_error_message = None;
        if let Some(subscription) = self.mapping.lock().expect(FAILED_TO_LOCK).get_mut(topic) {
            let publisher_set: BTreeSet<String> = publishers.collect();
            subscription.limit_publishers_to(&publisher_set);
            for publisher in publisher_set {
                if let Err(err) = connect_to_publisher(subscription, name, &publisher, topic) {
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
            .map(SubscriberRosConnection::get_topic)
            .cloned()
            .collect()
    }

    pub fn add<T, H>(&self, name: &str, topic: &str, queue_size: usize, handler: H) -> Result<usize>
    where
        T: Message,
        H: SubscriptionHandler<T>,
    {
        let msg_definition = T::msg_definition();
        let msg_type = T::msg_type();
        let md5sum = T::md5sum();
        let mut mapping = self.mapping.lock().expect(FAILED_TO_LOCK);
        let connection = mapping.entry(String::from(topic)).or_insert_with(|| {
            SubscriberRosConnection::new(
                name,
                topic,
                msg_definition,
                msg_type.clone(),
                md5sum.clone(),
            )
        });
        let connection_topic = connection.get_topic();
        if !header_matches(&connection_topic.msg_type, &msg_type)
            || !header_matches(&connection_topic.md5sum, &md5sum)
        {
            error!(
                "Attempted to connect to {} topic '{}' with message type {}",
                connection_topic.msg_type, topic, msg_type
            );
            Err(ErrorKind::MismatchedType(
                topic.into(),
                connection_topic.msg_type.clone(),
                msg_type,
            )
            .into())
        } else {
            Ok(connection.add_subscriber(queue_size, handler))
        }
    }

    #[inline]
    pub fn remove(&self, topic: &str, id: usize) {
        let mut mapping = self.mapping.lock().expect(FAILED_TO_LOCK);
        let has_subs = match mapping.get_mut(topic) {
            None => return,
            Some(val) => {
                val.remove_subscriber(id);
                val.has_subscribers()
            }
        };
        if !has_subs {
            mapping.remove(topic);
        }
    }

    #[inline]
    pub fn publisher_count(&self, topic: &str) -> usize {
        self.mapping
            .lock()
            .expect(FAILED_TO_LOCK)
            .get(topic)
            .map_or(0, SubscriberRosConnection::publisher_count)
    }

    #[inline]
    pub fn publisher_uris(&self, topic: &str) -> Vec<String> {
        self.mapping
            .lock()
            .expect(FAILED_TO_LOCK)
            .get(topic)
            .map_or_else(Vec::new, SubscriberRosConnection::publisher_uris)
    }
}

fn header_matches(first: &str, second: &str) -> bool {
    first == "*" || second == "*" || first == second
}

fn connect_to_publisher(
    subscriber: &mut SubscriberRosConnection,
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
            (caller_id, topic, [["TCPROS"]]),
        )
        .chain_err(|| error::rosxmlrpc::ErrorKind::TopicConnectionError(topic.to_owned()))?
        .map_err(|_| "error")?;
    Ok(protocols)
}
