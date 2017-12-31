use std::net::{TcpListener, ToSocketAddrs};
use std::thread;
use std::collections::HashMap;
use std;
use serde_rosmsg::to_vec;
use super::error::{ErrorKind, Result, ResultExt};
use super::header;
use super::Message;
use super::util::streamfork::{fork, DataStream, TargetList};
use super::util::tcpconnection;

pub struct Publisher {
    subscriptions: DataStream,
    pub port: u16,
    pub msg_type: String,
    pub topic: String,
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

fn listen_for_subscribers<T, U, V>(topic: &str, listener: V, targets: &TargetList<U>)
where
    T: Message,
    U: std::io::Read + std::io::Write + Send,
    V: Iterator<Item = U>,
{
    // This listener stream never breaks by itself since it's a TcpListener
    for mut stream in listener {
        let result = exchange_headers::<T, _>(&mut stream, topic)
            .chain_err(|| ErrorKind::TopicConnectionFail(topic.into()));
        if let Err(err) = result {
            let info = err.iter()
                .map(|v| format!("{}", v))
                .collect::<Vec<_>>()
                .join("\nCaused by:");
            error!("{}", info);
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
        thread::spawn(move || listen_for_subscribers::<T, _, _>(&topic_name, listener, &targets));
        Publisher {
            subscriptions: data,
            port: port,
            msg_type: T::msg_type(),
            topic: String::from(topic),
            _raii: raii,
        }
    }

    pub fn stream<T: Message>(&self) -> Result<PublisherStream<T>> {
        PublisherStream::new(self)
    }
}

// TODO: publisher should only be removed from master API once the publisher and all
// publisher streams are gone. This should be done with a RAII Arc, residing next todo
// the datastream. So maybe replace DataStream with a wrapper that holds that Arc too

#[derive(Clone)]
pub struct PublisherStream<T: Message> {
    stream: DataStream,
    datatype: std::marker::PhantomData<T>,
}

impl<T: Message> PublisherStream<T> {
    fn new(publisher: &Publisher) -> Result<PublisherStream<T>> {
        let msg_type = T::msg_type();
        if publisher.msg_type != msg_type {
            bail!(ErrorKind::MessageTypeMismatch(
                publisher.msg_type.clone(),
                msg_type,
            ));
        }
        Ok(PublisherStream {
            stream: publisher.subscriptions.clone(),
            datatype: std::marker::PhantomData,
        })
    }

    pub fn send(&mut self, message: &T) -> Result<()> {
        let bytes = to_vec(message)?;
        // Subscriptions can only be closed from the Publisher side
        // There is no way for the streamfork thread to fail by itself
        self.stream.send(bytes).expect("Connected thread died");
        Ok(())
    }
}
