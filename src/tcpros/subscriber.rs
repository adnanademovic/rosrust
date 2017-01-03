use std::net::{SocketAddr, TcpStream, ToSocketAddrs};
use std::sync::mpsc::{channel, Sender, Receiver};
use std::thread;
use std::collections::HashMap;
use std;
use super::error::{ErrorKind, Result, ResultExt};
use super::header::{encode, decode, match_field};
use super::Message;
use super::decoder::{Decoder, DecoderSource};

pub struct Subscriber {
    data_stream: Sender<Option<Decoder>>,
    publishers_stream: Sender<SocketAddr>,
    pub topic: String,
    pub msg_type: String,
}

impl Subscriber {
    pub fn new<T, F>(caller_id: &str, topic: &str, callback: F) -> Subscriber
        where T: Message,
              F: Fn(T) -> () + Send + 'static
    {
        let (data_tx, data_rx) = channel();
        let (pub_tx, pub_rx) = channel();
        let caller_id = String::from(caller_id);
        let topic_name = String::from(topic);
        let data_stream = data_tx.clone();
        thread::spawn(move || join_connections::<T>(data_tx, pub_rx, &caller_id, &topic_name));
        thread::spawn(move || handle_data::<T, F>(data_rx, callback));
        Subscriber {
            data_stream: data_stream,
            publishers_stream: pub_tx,
            topic: String::from(topic),
            msg_type: T::msg_type(),
        }
    }

    pub fn connect_to<U: ToSocketAddrs>(&mut self, addresses: U) -> std::io::Result<()> {
        for address in addresses.to_socket_addrs()? {
            // This should never fail, so it's safe to unwrap
            // Failure could only be caused by the join_connections
            // thread not running, which only happens after
            // Subscriber has been deconstructed
            self.publishers_stream.send(address).expect("Connected thread died");
        }
        Ok(())
    }
}

impl Drop for Subscriber {
    fn drop(&mut self) {
        if self.data_stream.send(None).is_err() {
            error!("Subscriber data stream to topic '{}' has already been killed",
                   self.topic);
        }
    }
}

fn handle_data<T, F>(data: Receiver<Option<Decoder>>, callback: F)
    where T: Message,
          F: Fn(T) -> ()
{
    for decoder_option in data {
        let mut decoder = match decoder_option {
            Some(v) => v,
            None => break, // Only the Subscriber destructor can send this signal
        };
        match T::decode(&mut decoder) {
            Ok(value) => callback(value),
            Err(err) => error!("Failed to decode message: {}", err),
        }
    }
}

fn join_connections<T>(data_stream: Sender<Option<Decoder>>,
                       publishers: Receiver<SocketAddr>,
                       caller_id: &str,
                       topic: &str)
    where T: Message
{
    // Ends when publisher sender is destroyed, which happens at Subscriber destruction
    for publisher in publishers {
        let result = join_connection::<T>(&data_stream, &publisher, caller_id, topic)
            .chain_err(|| ErrorKind::TopicConnectionFail(topic.into()));
        if let Err(err) = result {
            let info =
                err.iter().map(|v| format!("{}", v)).collect::<Vec<_>>().join("\nCaused by:");
            error!("{}", info);
        }
    }
}

fn join_connection<T>(data_stream: &Sender<Option<Decoder>>,
                      publisher: &SocketAddr,
                      caller_id: &str,
                      topic: &str)
                      -> Result<()>
    where T: Message
{
    let mut stream = TcpStream::connect(publisher)?;
    exchange_headers::<T, _>(&mut stream, caller_id, topic)?;
    let target = data_stream.clone();
    thread::spawn(move || {
        for decoder in DecoderSource::new(stream) {
            if target.send(Some(decoder)).is_err() {
                // Data receiver has been destroyed after Subscriber destructor's kill signal
                break;
            }
        }
    });
    Ok(())
}

fn write_request<T: Message, U: std::io::Write>(mut stream: &mut U,
                                                caller_id: &str,
                                                topic: &str)
                                                -> Result<()> {
    let mut fields = HashMap::<String, String>::new();
    fields.insert(String::from("message_definition"), T::msg_definition());
    fields.insert(String::from("callerid"), String::from(caller_id));
    fields.insert(String::from("topic"), String::from(topic));
    fields.insert(String::from("md5sum"), T::md5sum());
    fields.insert(String::from("type"), T::msg_type());
    encode(fields)?.write_to(&mut stream)?;
    Ok(())
}

fn read_response<T: Message, U: std::io::Read>(mut stream: &mut U) -> Result<()> {
    let fields = decode(&mut stream)?;
    match_field(&fields, "md5sum", &T::md5sum())?;
    match_field(&fields, "type", &T::msg_type())
}

fn exchange_headers<T, U>(mut stream: &mut U, caller_id: &str, topic: &str) -> Result<()>
    where T: Message,
          U: std::io::Write + std::io::Read
{
    write_request::<T, U>(stream, caller_id, topic)?;
    read_response::<T, U>(stream)
}
