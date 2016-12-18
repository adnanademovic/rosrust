use rustc_serialize::Encodable;
use std::clone::Clone;
use std::net::{TcpListener, TcpStream, ToSocketAddrs};
use std::sync::mpsc;
use std::thread;
use std;
use super::encoder::Encoder;
use super::error::Error;
use super::header::{encode, decode};
use super::Message;

pub struct Publisher<T>
    where T: Message + Encodable + Clone + Send + 'static
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
    where T: Message + Encodable + Clone + Send
{
    let caller_id: String;
    {
        let fields = decode(&mut stream)?;
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
        encode(fields, &mut stream)?;
    }
    while let Ok(v) = rx.recv() {
        let mut encoder = Encoder::new();
        v.encode(&mut encoder)?;
        encoder.write_to(&mut stream)?;
    }
    Ok(())
}

fn listen<T>(topic: String,
             listener: TcpListener,
             tx_sender: mpsc::Sender<mpsc::Sender<T>>)
             -> Result<(), Error>
    where T: Message + Encodable + Clone + Send + 'static
{
    for stream in listener.incoming() {
        let stream = stream?;
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
    where T: Message + Encodable + Clone + Send + 'static
{
    pub fn new<U>(address: U, topic: &str) -> Result<Publisher<T>, Error>
        where U: ToSocketAddrs
    {
        let listener = TcpListener::bind(address)?;
        let (tx_sender, rx_sender) = mpsc::channel();
        let socket_address = listener.local_addr()?;
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
