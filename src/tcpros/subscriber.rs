use rustc_serialize::Decodable;
use std::net::{TcpStream, ToSocketAddrs};
use std::sync::mpsc;
use std::thread;
use std::collections::HashMap;
use std;
use super::error::Error;
use super::header::{encode, decode};
use super::Message;
use super::decoder::DecoderSource;

pub struct Subscriber<T>
    where T: Message + Decodable + Send + 'static
{
    message_stream: mpsc::Receiver<T>,
}

impl<T> Subscriber<T>
    where T: Message + Decodable + Send + 'static
{
    pub fn new<U>(address: U, caller_id: &str, topic: &str) -> Result<Subscriber<T>, Error>
        where U: ToSocketAddrs
    {
        Subscriber::<T>::wrap_stream(TcpStream::connect(address)?, caller_id, topic)
    }

    fn wrap_stream<U>(mut stream: U, caller_id: &str, topic: &str) -> Result<Subscriber<T>, Error>
        where U: std::io::Read + std::io::Write + Send + 'static
    {
        write_request::<T, U>(&mut stream, caller_id, topic)?;
        if !header_matches::<T>(&decode(&mut stream)?) {
            return Err(Error::Mismatch);
        }

        let (tx_message_stream, rx_message_stream) = mpsc::channel();

        thread::spawn(move || decode_stream(stream, tx_message_stream));

        Ok(Subscriber { message_stream: rx_message_stream })
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
    encode(fields, &mut stream)
}

fn header_matches<T: Message>(fields: &HashMap<String, String>) -> bool {
    fields.get("md5sum") == Some(&T::md5sum()) && fields.get("type") == Some(&T::msg_type())
}

fn decode_stream<T, U>(stream: U, message_sender: mpsc::Sender<T>) -> Result<(), Error>
    where T: Message + Decodable,
          U: std::io::Read
{
    for mut decoder in DecoderSource::new(stream) {
        if message_sender.send(T::decode(&mut decoder)?).is_err() {
            break;
        }
    }
    Ok(())
}

impl<T> std::iter::Iterator for Subscriber<T>
    where T: Message + Decodable + Send + 'static
{
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        self.message_stream.recv().ok()
    }
}
