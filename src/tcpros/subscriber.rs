use std::net::{SocketAddr, TcpStream, ToSocketAddrs};
use std::sync::mpsc::{channel, Sender, Receiver};
use std::thread;
use std::collections::HashMap;
use std;
use super::error::Error;
use super::header::{encode, decode, match_field};
use super::Message;
use super::decoder::{Decoder, DecoderSource};

pub struct Subscriber {
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
        thread::spawn(move || join_connections::<T>(data_tx, pub_rx, &caller_id, &topic_name));
        thread::spawn(move || handle_data::<T, F>(data_rx, callback));
        Subscriber {
            publishers_stream: pub_tx,
            topic: String::from(topic),
            msg_type: T::msg_type(),
        }
    }

    pub fn connect_to<U: ToSocketAddrs>(&mut self, addresses: U) -> std::io::Result<()> {
        for address in addresses.to_socket_addrs()? {
            // This should never fail, so it's safe to unwrap
            // Failure could only be caused by the join_connections
            // thread not running, which should never happen
            self.publishers_stream.send(address).unwrap();
        }
        Ok(())
    }
}

fn handle_data<T, F>(data: Receiver<Decoder>, callback: F)
    where T: Message,
          F: Fn(T) -> ()
{
    for mut decoder in data {
        match T::decode(&mut decoder) {
            Ok(value) => callback(value),
            Err(err) => error!("Failed to decode message: {}", err),
        }
    }
}

fn join_connections<T>(data_stream: Sender<Decoder>,
                       publishers: Receiver<SocketAddr>,
                       caller_id: &str,
                       topic: &str)
    where T: Message
{
    for publisher in publishers {
        let mut stream = match TcpStream::connect(publisher) {
            Ok(stream) => stream,
            Err(err) => {
                error!("Failed to subscribe to topic '{}': {}", topic, err);
                continue;
            }
        };
        if let Err(err) = exchange_headers::<T, _>(&mut stream, caller_id, topic) {
            error!("Headers mismatched while subscribing to topic '{}': {}",
                   topic,
                   err);
            continue;
        }
        let target = data_stream.clone();
        thread::spawn(move || {
            for decoder in DecoderSource::new(stream) {
                if let Err(_) = target.send(decoder) {
                    break;
                }
            }
        });
    }
}

fn write_request<T: Message, U: std::io::Write>(mut stream: &mut U,
                                                caller_id: &str,
                                                topic: &str)
                                                -> Result<(), Error> {
    let mut fields = HashMap::<String, String>::new();
    fields.insert(String::from("message_definition"), T::msg_definition());
    fields.insert(String::from("callerid"), String::from(caller_id));
    fields.insert(String::from("topic"), String::from(topic));
    fields.insert(String::from("md5sum"), T::md5sum());
    fields.insert(String::from("type"), T::msg_type());
    encode(fields)?.write_to(&mut stream)?;
    Ok(())
}

fn read_response<T: Message, U: std::io::Read>(mut stream: &mut U) -> Result<(), Error> {
    let fields = decode(&mut stream)?;
    match_field(&fields, "md5sum", &T::md5sum())?;
    match_field(&fields, "type", &T::msg_type())
}

fn exchange_headers<T, U>(mut stream: &mut U, caller_id: &str, topic: &str) -> Result<(), Error>
    where T: Message,
          U: std::io::Write + std::io::Read
{
    write_request::<T, U>(stream, caller_id, topic)?;
    read_response::<T, U>(stream)
}
