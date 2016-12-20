use rustc_serialize::Encodable;
use std::clone::Clone;
use std::net::{TcpListener, TcpStream, ToSocketAddrs};
use std::sync::mpsc;
use std::thread;
use std::collections::HashMap;
use std;
use super::encoder::Encoder;
use super::error::Error;
use super::header::{encode, decode};
use super::Message;

pub struct Publisher<T>
    where T: Message + Encodable + Clone + Send + 'static
{
    subscription_requests: mpsc::Receiver<mpsc::Sender<T>>,
    subscriptions: Vec<mpsc::Sender<T>>,
    pub ip: String,
    pub port: u16,
}

fn handle_stream<T>(topic: String,
                    mut stream: TcpStream,
                    rx: mpsc::Receiver<T>)
                    -> Result<(), Error>
    where T: Message + Encodable + Clone + Send
{
    if !header_matches::<T>(&decode(&mut stream)?, &topic) {
        return Err(Error::Mismatch);
    }
    write_response::<T, TcpStream>(&mut stream)?;

    while let Ok(v) = rx.recv() {
        let mut encoder = Encoder::new();
        v.encode(&mut encoder)?;
        encoder.write_to(&mut stream)?;
    }
    Ok(())
}

fn header_matches<T: Message>(fields: &HashMap<String, String>, topic: &String) -> bool {
    fields.get("md5sum") == Some(&T::md5sum()) && fields.get("type") == Some(&T::msg_type()) &&
    fields.get("message_definition") == Some(&T::msg_definition()) &&
    fields.get("topic") == Some(topic) && fields.get("callerid") != None
}

fn write_response<T: Message, U: std::io::Write>(mut stream: &mut U) -> Result<(), Error> {
    let mut fields = HashMap::<String, String>::new();
    fields.insert(String::from("md5sum"), T::md5sum());
    fields.insert(String::from("type"), T::msg_type());
    encode(fields, &mut stream)
}

fn listen<T>(topic: String,
             listener: TcpListener,
             subscription_requests: mpsc::Sender<mpsc::Sender<T>>)
             -> Result<(), Error>
    where T: Message + Encodable + Clone + Send + 'static
{
    for stream in listener.incoming() {
        let stream = stream?;
        let (tx, rx) = mpsc::channel();
        if let Err(_) = subscription_requests.send(tx) {
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
        let (tx_subscription_requests, rx_subscription_requests) = mpsc::channel();
        let socket_address = listener.local_addr()?;
        let topic = String::from(topic);
        thread::spawn(move || listen(topic, listener, tx_subscription_requests));
        Ok(Publisher {
            subscription_requests: rx_subscription_requests,
            subscriptions: Vec::new(),
            ip: format!("{}", socket_address.ip()),
            port: socket_address.port(),
        })
    }

    pub fn send(&mut self, message: T) {
        while let Ok(subscriber) = self.subscription_requests.try_recv() {
            self.subscriptions.push(subscriber);
        }
        // Attempt to send and filter out connections that were stopped
        self.subscriptions = self.subscriptions
            .clone()
            .into_iter()
            .filter(|subscriber| subscriber.send(message.clone()).is_ok())
            .collect();
    }
}
