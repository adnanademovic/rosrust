use std::net::{TcpListener, TcpStream, ToSocketAddrs};
use std::thread;
use std::collections::HashMap;
use std;
use super::encoder::Encoder;
use super::error::{Error, ErrorKind};
use super::header::{encode, decode, match_field};
use super::Message;
use super::streamfork::{fork, TargetList, DataStream};

pub struct Publisher {
    subscriptions: DataStream,
    pub port: u16,
    pub msg_type: String,
    pub topic: String,
}

fn read_request<T: Message, U: std::io::Read>(mut stream: &mut U,
                                              topic: &str)
                                              -> Result<(), Error> {
    let fields = decode(&mut stream)?;
    match_field(&fields, "md5sum", &T::md5sum())?;
    match_field(&fields, "type", &T::msg_type())?;
    match_field(&fields, "message_definition", &T::msg_definition())?;
    match_field(&fields, "topic", topic)?;
    if fields.get("callerid").is_none() {
        bail!(ErrorKind::HeaderMissingField("callerid".into()));
    }
    Ok(())
}

fn write_response<T: Message, U: std::io::Write>(mut stream: &mut U) -> Result<(), Error> {
    let mut fields = HashMap::<String, String>::new();
    fields.insert(String::from("md5sum"), T::md5sum());
    fields.insert(String::from("type"), T::msg_type());
    encode(fields)?.write_to(&mut stream)?;
    Ok(())
}

fn exchange_headers<T, U>(mut stream: &mut U, topic: &str) -> Result<(), Error>
    where T: Message,
          U: std::io::Write + std::io::Read
{
    read_request::<T, U>(&mut stream, &topic)?;
    write_response::<T, U>(&mut stream)
}

fn listen_for_subscribers<T, U, V>(topic: String,
                                   listener: V,
                                   targets: TargetList<U>)
                                   -> Result<(), Error>
    where T: Message,
          U: std::io::Read + std::io::Write + Send,
          V: Iterator<Item = U>
{
    for mut stream in listener {
        if let Err(err) = exchange_headers::<T, _>(&mut stream, &topic) {
            error!("Failed to exchange headers for topic '{}': {}", topic, err);
            continue;
        }
        if targets.add(stream).is_err() {
            // The TCP listener gets shut down when streamfork's thread deallocates
            // This happens only when the corresponding Publisher gets deallocated,
            // causing streamfork's data channel to shut down
            break;
        }
    }

    Ok(())
}

impl Publisher {
    pub fn new<T, U>(address: U, topic: &str) -> Result<Publisher, Error>
        where T: Message,
              U: ToSocketAddrs
    {
        let listener = TcpListener::bind(address)?;
        let socket_address = listener.local_addr()?;
        Ok(Publisher::wrap_stream::<T, _, _>(topic,
                                             TcpIterator::new(listener, topic),
                                             socket_address.port()))
    }

    fn wrap_stream<T, U, V>(topic: &str, listener: V, port: u16) -> Publisher
        where T: Message,
              U: std::io::Read + std::io::Write + Send + 'static,
              V: Iterator<Item = U> + Send + 'static
    {
        let (targets, data) = fork();
        let topic_name = String::from(topic);
        thread::spawn(move || listen_for_subscribers::<T, _, _>(topic_name, listener, targets));
        Publisher {
            subscriptions: data,
            port: port,
            msg_type: T::msg_type(),
            topic: String::from(topic),
        }
    }

    pub fn stream<T: Message>(&self) -> Result<PublisherStream<T>, Error> {
        PublisherStream::new(self)
    }
}

pub struct PublisherStream<T: Message> {
    stream: DataStream,
    datatype: std::marker::PhantomData<T>,
}

impl<T: Message> PublisherStream<T> {
    fn new(publisher: &Publisher) -> Result<PublisherStream<T>, Error> {
        let msg_type = T::msg_type();
        if publisher.msg_type != msg_type {
            bail!(ErrorKind::MessageTypeMismatch(publisher.msg_type.clone(), msg_type));
        }
        Ok(PublisherStream {
            stream: publisher.subscriptions.clone(),
            datatype: std::marker::PhantomData,
        })
    }

    pub fn send(&mut self, message: T) {
        let mut encoder = Encoder::new();
        // Failure while encoding can only be caused by unsupported data types,
        // unless using deliberately bad handwritten rosmsg-s, this should never fail
        message.encode(&mut encoder).unwrap();
        // Subscriptions can only be closed from the Publisher side
        // There is no way for the streamfork thread to fail by itself
        self.stream.send(encoder).unwrap();
    }
}

struct TcpIterator {
    listener: TcpListener,
    topic: String,
}

impl TcpIterator {
    pub fn new(listener: TcpListener, topic: &str) -> TcpIterator {
        TcpIterator {
            listener: listener,
            topic: String::from(topic),
        }
    }
}

impl Iterator for TcpIterator {
    type Item = TcpStream;

    fn next(&mut self) -> Option<Self::Item> {
        match self.listener.accept() {
            Ok((stream, _)) => Some(stream),
            Err(err) => {
                error!("TCP connection to subscriber failed on topic '{}': {}",
                       self.topic,
                       err);
                self.next()
            }
        }
    }
}
