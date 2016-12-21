use rustc_serialize::Encodable;
use std::clone::Clone;
use std::net::{TcpListener, TcpStream, ToSocketAddrs};
use std::sync::{mpsc, Arc};
use std::thread;
use std::collections::HashMap;
use std;
use super::encoder::Encoder;
use super::error::Error;
use super::header::{encode, decode};
use super::Message;

pub struct Publisher {
    subscription_requests: mpsc::Receiver<mpsc::Sender<Arc<Encoder>>>,
    subscriptions: Vec<mpsc::Sender<Arc<Encoder>>>,
    pub ip: String,
    pub port: u16,
}

fn handle_stream(mut stream: TcpStream, rx: mpsc::Receiver<Arc<Encoder>>) -> Result<(), Error> {
    while let Ok(encoder) = rx.recv() {
        encoder.write_to(&mut stream)?;
    }
    Ok(())
}

fn header_matches<T: Message>(fields: &HashMap<String, String>, topic: &str) -> bool {
    fields.get("md5sum") == Some(&T::md5sum()) && fields.get("type") == Some(&T::msg_type()) &&
    fields.get("message_definition") == Some(&T::msg_definition()) &&
    fields.get("topic") == Some(&String::from(topic)) && fields.get("callerid") != None
}

fn read_request<T: Message, U: std::io::Read>(mut stream: &mut U,
                                              topic: &str)
                                              -> Result<(), Error> {
    if header_matches::<T>(&decode(&mut stream)?, topic) {
        Ok(())
    } else {
        Err(Error::Mismatch)
    }
}

fn write_response<T: Message, U: std::io::Write>(mut stream: &mut U) -> Result<(), Error> {
    let mut fields = HashMap::<String, String>::new();
    fields.insert(String::from("md5sum"), T::md5sum());
    fields.insert(String::from("type"), T::msg_type());
    encode(fields, &mut stream)
}

fn exchange_headers<T, U>(mut stream: &mut U, topic: &str) -> bool
    where T: Message,
          U: std::io::Write + std::io::Read
{
    if read_request::<T, U>(&mut stream, &topic).is_err() {
        false
    } else {
        write_response::<T, U>(&mut stream).is_ok()
    }
}

fn listen<T: Message>(topic: String,
                      listener: TcpListener,
                      subscription_requests: mpsc::Sender<mpsc::Sender<Arc<Encoder>>>)
                      -> Result<(), Error> {
    for stream in listener.incoming() {
        let mut stream = stream?;
        let (tx, rx) = mpsc::channel();
        if let Err(_) = subscription_requests.send(tx) {
            // Stop once the corresponding Publisher gets destroyed
            break;
        }
        if !exchange_headers::<T, _>(&mut stream, &topic) {
            continue;
        }
        thread::spawn(move || handle_stream(stream, rx));
    }
    Ok(())
}

impl Publisher {
    pub fn new<T, U>(address: U, topic: &str) -> Result<Publisher, Error>
        where T: Message,
              U: ToSocketAddrs
    {
        let listener = TcpListener::bind(address)?;
        let (tx_subscription_requests, rx_subscription_requests) = mpsc::channel();
        let socket_address = listener.local_addr()?;
        let topic = String::from(topic);
        thread::spawn(move || listen::<T>(topic, listener, tx_subscription_requests));
        Ok(Publisher {
            subscription_requests: rx_subscription_requests,
            subscriptions: Vec::new(),
            ip: format!("{}", socket_address.ip()),
            port: socket_address.port(),
        })
    }

    pub fn send<T>(&mut self, message: T)
        where T: Message + Encodable
    {
        while let Ok(subscriber) = self.subscription_requests.try_recv() {
            self.subscriptions.push(subscriber);
        }
        let mut encoder = Encoder::new();
        message.encode(&mut encoder).unwrap();
        let encoder = Arc::new(encoder);
        self.subscriptions.retain(|subscriber| subscriber.send(encoder.clone()).is_ok());
    }
}
