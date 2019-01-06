use super::error::{ErrorKind, Result, ResultExt};
use super::header;
use super::util::streamfork::{fork, DataStream, TargetList};
use super::util::tcpconnection;
use super::{Message, Topic};
use std;
use std::collections::HashMap;
use std::net::{TcpListener, ToSocketAddrs};
use std::sync::{Arc, Mutex};
use std::thread;

pub struct Publisher {
    subscriptions: DataStream,
    pub port: u16,
    pub topic: Topic,
    last_message: Arc<Mutex<Arc<Vec<u8>>>>,
    _raii: tcpconnection::Raii,
}

fn read_request<T: Message, U: std::io::Read>(mut stream: &mut U, topic: &str) -> Result<()> {
    let fields = header::decode(&mut stream)?;
    header::match_field(&fields, "md5sum", &T::md5sum())?;
    header::match_field(&fields, "type", &T::msg_type())?;
    header::match_field(&fields, "topic", topic)?;
    if fields.get("callerid").is_none() {
        bail!(ErrorKind::HeaderMissingField("callerid".into()));
    }
    Ok(())
}

fn write_response<T: Message, U: std::io::Write>(mut stream: &mut U) -> Result<()> {
    let mut fields = HashMap::<String, String>::new();
    fields.insert(String::from("md5sum"), T::md5sum());
    fields.insert(String::from("type"), T::msg_type());
    header::encode(&mut stream, &fields)?;
    Ok(())
}

fn exchange_headers<T, U>(mut stream: &mut U, topic: &str) -> Result<()>
where
    T: Message,
    U: std::io::Write + std::io::Read,
{
    read_request::<T, U>(&mut stream, topic)?;
    write_response::<T, U>(&mut stream)
}

fn listen_for_subscribers<T, U, V>(
    topic: &str,
    listener: V,
    targets: &TargetList<U>,
    last_message: &Mutex<Arc<Vec<u8>>>,
) where
    T: Message,
    U: std::io::Read + std::io::Write + Send,
    V: Iterator<Item = U>,
{
    // This listener stream never breaks by itself since it's a TcpListener
    for mut stream in listener {
        let result = exchange_headers::<T, _>(&mut stream, topic)
            .chain_err(|| ErrorKind::TopicConnectionFail(topic.into()));
        if let Err(err) = result {
            let info = err
                .iter()
                .map(|v| format!("{}", v))
                .collect::<Vec<_>>()
                .join("\nCaused by:");
            error!("{}", info);
            continue;
        }

        if let Err(err) = stream.write_all(&last_message.lock().unwrap()) {
            error!("{}", err);
            continue;
        }

        if targets.add(stream).is_err() {
            // The TCP listener gets shut down when streamfork's thread deallocates.
            // This happens only when all the corresponding publisher streams get deallocated,
            // causing streamfork's data channel to shut down
            break;
        }
    }
}

impl Publisher {
    pub fn new<T, U>(address: U, topic: &str) -> Result<Publisher>
    where
        T: Message,
        U: ToSocketAddrs,
    {
        let listener = TcpListener::bind(address)?;
        let socket_address = listener.local_addr()?;
        let (raii, listener) = tcpconnection::iterate(listener, format!("topic '{}'", topic));
        Ok(Publisher::wrap_stream::<T, _, _>(
            topic,
            raii,
            listener,
            socket_address.port(),
        ))
    }

    fn wrap_stream<T, U, V>(
        topic: &str,
        raii: tcpconnection::Raii,
        listener: V,
        port: u16,
    ) -> Publisher
    where
        T: Message,
        U: std::io::Read + std::io::Write + Send + 'static,
        V: Iterator<Item = U> + Send + 'static,
    {
        let (targets, data) = fork();
        let topic_name = String::from(topic);
        let last_message = Arc::new(Mutex::new(Arc::new(Vec::new())));
        let last_msg_for_thread = Arc::clone(&last_message);
        thread::spawn(move || {
            listen_for_subscribers::<T, _, _>(&topic_name, listener, &targets, &last_msg_for_thread)
        });
        let topic = Topic {
            name: String::from(topic),
            msg_type: T::msg_type(),
        };
        Publisher {
            subscriptions: data,
            port,
            topic,
            last_message,
            _raii: raii,
        }
    }

    pub fn stream<T: Message>(&self) -> Result<PublisherStream<T>> {
        PublisherStream::new(self)
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
        Ok(PublisherStream {
            stream: publisher.subscriptions.clone(),
            datatype: std::marker::PhantomData,
            last_message: Arc::clone(&publisher.last_message),
            latching: false,
        })
    }

    #[inline]
    pub fn set_latching(&mut self, latching: bool) {
        self.latching = latching;
    }

    #[inline]
    pub fn set_queue_size(&mut self, queue_size: Option<usize>) {
        self.stream.set_queue_size(queue_size);
    }

    pub fn send(&mut self, message: &T) -> Result<()> {
        let bytes = Arc::new(message.encode_vec()?);

        if self.latching {
            *self.last_message.lock().unwrap() = Arc::clone(&bytes);
        }

        // Subscriptions can only be closed from the Publisher side
        // There is no way for the streamfork thread to fail by itself
        self.stream.send(bytes).expect("Connected thread died");
        Ok(())
    }
}
