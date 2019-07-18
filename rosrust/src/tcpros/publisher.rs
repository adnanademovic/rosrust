use super::error::{ErrorKind, Result, ResultExt};
use super::header;
use super::util::streamfork::{fork, DataStream, TargetList};
use super::util::tcpconnection;
use super::{Message, Topic};
use crate::util::FAILED_TO_LOCK;
use log::error;
use std;
use std::collections::HashMap;
use std::net::{TcpListener, TcpStream, ToSocketAddrs};
use std::sync::{atomic, Arc, Mutex};

pub struct Publisher {
    subscriptions: DataStream,
    pub port: u16,
    pub topic: Topic,
    last_message: Arc<Mutex<Arc<Vec<u8>>>>,
    queue_size: usize,
    exists: Arc<atomic::AtomicBool>,
}

impl Drop for Publisher {
    fn drop(&mut self) {
        self.exists.store(false, atomic::Ordering::SeqCst);
    }
}

fn match_headers<T: Message>(fields: &HashMap<String, String>, topic: &str) -> Result<()> {
    header::match_field(fields, "md5sum", &T::md5sum())
        .or_else(|e| header::match_field(fields, "md5sum", "*").or(Err(e)))?;
    header::match_field(fields, "type", &T::msg_type())
        .or_else(|e| header::match_field(fields, "type", "*").or(Err(e)))?;
    header::match_field(fields, "topic", topic)?;
    Ok(())
}

fn read_request<T: Message, U: std::io::Read>(mut stream: &mut U, topic: &str) -> Result<String> {
    let fields = header::decode(&mut stream)?;
    match_headers::<T>(&fields, topic)?;
    let caller_id = fields
        .get("callerid")
        .ok_or_else(|| ErrorKind::HeaderMissingField("callerid".into()))?;
    Ok(caller_id.clone())
}

fn write_response<T: Message, U: std::io::Write>(
    mut stream: &mut U,
    caller_id: &str,
) -> Result<()> {
    let mut fields = HashMap::<String, String>::new();
    fields.insert(String::from("md5sum"), T::md5sum());
    fields.insert(String::from("type"), T::msg_type());
    fields.insert(String::from("callerid"), caller_id.into());
    header::encode(&mut stream, &fields)?;
    Ok(())
}

fn exchange_headers<T, U>(mut stream: &mut U, topic: &str, pub_caller_id: &str) -> Result<String>
where
    T: Message,
    U: std::io::Write + std::io::Read,
{
    let caller_id = read_request::<T, U>(&mut stream, topic)?;
    write_response::<T, U>(&mut stream, pub_caller_id)?;
    Ok(caller_id)
}

fn process_subscriber<T, U>(
    topic: &str,
    mut stream: U,
    targets: &TargetList<U>,
    last_message: &Mutex<Arc<Vec<u8>>>,
    pub_caller_id: &str,
) -> tcpconnection::Feedback
where
    T: Message,
    U: std::io::Read + std::io::Write + Send,
{
    let result = exchange_headers::<T, _>(&mut stream, topic, pub_caller_id)
        .chain_err(|| ErrorKind::TopicConnectionFail(topic.into()));
    let caller_id = match result {
        Ok(caller_id) => caller_id,
        Err(err) => {
            let info = err
                .iter()
                .map(|v| format!("{}", v))
                .collect::<Vec<_>>()
                .join("\nCaused by:");
            error!("{}", info);
            return tcpconnection::Feedback::AcceptNextStream;
        }
    };

    if let Err(err) = stream.write_all(&last_message.lock().expect(FAILED_TO_LOCK)) {
        error!("{}", err);
        return tcpconnection::Feedback::AcceptNextStream;
    }

    if targets.add(caller_id, stream).is_err() {
        // The TCP listener gets shut down when streamfork's thread deallocates.
        // This happens only when all the corresponding publisher streams get deallocated,
        // causing streamfork's data channel to shut down
        return tcpconnection::Feedback::StopAccepting;
    }

    tcpconnection::Feedback::AcceptNextStream
}

impl Publisher {
    pub fn new<T, U>(
        address: U,
        topic: &str,
        queue_size: usize,
        caller_id: &str,
    ) -> Result<Publisher>
    where
        T: Message,
        U: ToSocketAddrs,
    {
        let listener = TcpListener::bind(address)?;
        let socket_address = listener.local_addr()?;

        let publisher_exists = Arc::new(atomic::AtomicBool::new(true));

        let port = socket_address.port();
        let (targets, data) = fork(queue_size);
        let last_message = Arc::new(Mutex::new(Arc::new(Vec::new())));

        let iterate_handler = {
            let publisher_exists = publisher_exists.clone();
            let topic = String::from(topic);
            let last_message = Arc::clone(&last_message);
            let caller_id = String::from(caller_id);

            move |stream: TcpStream| {
                if !publisher_exists.load(atomic::Ordering::SeqCst) {
                    return tcpconnection::Feedback::StopAccepting;
                }
                process_subscriber::<T, _>(&topic, stream, &targets, &last_message, &caller_id)
            }
        };

        tcpconnection::iterate(listener, format!("topic '{}'", topic), iterate_handler);

        let topic = Topic {
            name: String::from(topic),
            msg_type: T::msg_type(),
        };

        Ok(Publisher {
            subscriptions: data,
            port,
            topic,
            last_message,
            queue_size,
            exists: publisher_exists,
        })
    }

    pub fn stream<T: Message>(&self, queue_size: usize) -> Result<PublisherStream<T>> {
        let mut stream = PublisherStream::new(self)?;
        stream.set_queue_size_max(queue_size);
        Ok(stream)
    }

    pub fn get_topic(&self) -> &Topic {
        &self.topic
    }
}

// TODO: publisher should only be removed from master API once the publisher and all
// publisher streams are gone. This should be done with a RAII Arc, residing next todo
// the datastream. So maybe replace DataStream with a wrapper that holds that Arc too

#[derive(Clone)]
pub struct PublisherStream<T: Message> {
    stream: DataStream,
    last_message: Arc<Mutex<Arc<Vec<u8>>>>,
    datatype: std::marker::PhantomData<T>,
    latching: bool,
}

impl<T: Message> PublisherStream<T> {
    fn new(publisher: &Publisher) -> Result<PublisherStream<T>> {
        let msg_type = T::msg_type();
        if publisher.topic.msg_type != msg_type {
            bail!(ErrorKind::MessageTypeMismatch(
                publisher.topic.msg_type.clone(),
                msg_type,
            ));
        }
        let mut stream = PublisherStream {
            stream: publisher.subscriptions.clone(),
            datatype: std::marker::PhantomData,
            last_message: Arc::clone(&publisher.last_message),
            latching: false,
        };
        stream.set_queue_size_max(publisher.queue_size);
        Ok(stream)
    }

    #[inline]
    pub fn subscriber_count(&self) -> usize {
        self.stream.target_count()
    }

    #[inline]
    pub fn subscriber_names(&self) -> Vec<String> {
        self.stream.target_names()
    }

    #[inline]
    pub fn set_latching(&mut self, latching: bool) {
        self.latching = latching;
    }

    #[inline]
    pub fn set_queue_size(&mut self, queue_size: usize) {
        self.stream.set_queue_size(queue_size);
    }

    #[inline]
    pub fn set_queue_size_max(&mut self, queue_size: usize) {
        self.stream.set_queue_size_max(queue_size);
    }

    pub fn send(&self, message: &T) -> Result<()> {
        let bytes = Arc::new(message.encode_vec()?);

        if self.latching {
            *self.last_message.lock().expect(FAILED_TO_LOCK) = Arc::clone(&bytes);
        }

        // Subscriptions can only be closed from the Publisher side
        // There is no way for the streamfork thread to fail by itself
        self.stream.send(bytes).expect("Connected thread died");
        Ok(())
    }
}
