use byteorder::{LittleEndian, ReadBytesExt};
use rustc_serialize::Encodable;
use std::clone::Clone;
use std::net::{TcpListener, TcpStream, ToSocketAddrs};
use std::io::{Read, Write};
use std::sync::mpsc;
use std::thread;
use std;
use super::encoder::Encoder;
use super::error::Error;
use super::header::{encode, decode};
use super::message::RosMessage;

pub struct Publisher<T>
    where T: RosMessage + Encodable + Clone + Send + 'static
{
    sender_receiver: mpsc::Receiver<mpsc::Sender<T>>,
    senders: Vec<mpsc::Sender<T>>,
    pub ip: String,
    pub port: u16,
}

fn handle_stream<T>(topic: String,
                    mut stream: TcpStream,
                    rx: mpsc::Receiver<T>)
                    -> Result<(), Error>
    where T: RosMessage + Encodable + Clone + Send
{
    let caller_id: String;
    {
        let mut bytes = [0u8; 4];
        try!(stream.read_exact(&mut bytes));
        let mut reader = std::io::Cursor::new(bytes);
        let data_length = try!(reader.read_u32::<LittleEndian>());
        let mut payload = vec![0u8; data_length as usize];
        try!(stream.read_exact(&mut payload));
        let data = bytes.iter().chain(payload.iter()).cloned().collect();
        let fields = try!(decode(data));
        if fields.get("md5sum") != Some(&T::md5sum()) {
            return Err(Error::Mismatch);
        }
        if fields.get("type") != Some(&T::msg_type()) {
            return Err(Error::Mismatch);
        }
        if fields.get("message_definition") != Some(&T::msg_definition()) {
            return Err(Error::Mismatch);
        }
        if fields.get("topic") != Some(&topic) {
            return Err(Error::Mismatch);
        }
        match fields.get("callerid") {
            None => return Err(Error::Mismatch),
            Some(v) => caller_id = v.to_owned(),
        }
    }
    // TODO: do something smarter with "caller_id" and "topic"
    {
        let mut fields = std::collections::HashMap::<String, String>::new();
        fields.insert("md5sum".to_owned(), T::md5sum());
        fields.insert("type".to_owned(), T::msg_type());
        let fields = try!(encode(fields));
        try!(stream.write_all(&fields));
    }
    while let Ok(v) = rx.recv() {
        let mut encoder = Encoder::new();
        try!(v.encode(&mut encoder));
        try!(stream.write_all(&encoder.extract_data()));
    }
    Ok(())
}

fn listen<T>(topic: String,
             listener: TcpListener,
             tx_sender: mpsc::Sender<mpsc::Sender<T>>)
             -> Result<(), Error>
    where T: RosMessage + Encodable + Clone + Send + 'static
{
    for stream in listener.incoming() {
        let stream = try!(stream);
        let (tx, rx) = mpsc::channel();
        if let Err(_) = tx_sender.send(tx) {
            // Stop once the corresponding Publisher gets destroyed
            break;
        }
        let topic = topic.clone();
        thread::spawn(move || handle_stream(topic, stream, rx));
    }
    Ok(())
}

impl<T> Publisher<T>
    where T: RosMessage + Encodable + Clone + Send + 'static
{
    pub fn new<U>(address: U, topic: &str) -> Result<Publisher<T>, Error>
        where U: ToSocketAddrs
    {
        let listener = try!(TcpListener::bind(address));
        let (tx_sender, rx_sender) = mpsc::channel();
        let socket_address = try!(listener.local_addr());
        let topic = topic.to_owned();
        thread::spawn(move || listen(topic, listener, tx_sender));
        Ok(Publisher {
            sender_receiver: rx_sender,
            senders: vec![],
            ip: format!("{}", socket_address.ip()),
            port: socket_address.port(),
        })
    }

    pub fn send(&mut self, message: T) {
        while let Ok(tx) = self.sender_receiver.try_recv() {
            self.senders.push(tx);
        }
        // Attempt to send and filter out connections that were stopped
        self.senders = self.senders
            .clone()
            .into_iter()
            .filter(|sender| sender.send(message.clone()).is_ok())
            .collect();
    }
}
