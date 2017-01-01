use rustc_serialize::{Encodable, Decodable};
use std::net::{TcpListener, TcpStream};
use std::thread;
use std::sync::Arc;
use std::collections::HashMap;
use std;
use super::decoder::DecoderSource;
use super::encoder::Encoder;
use super::error::{Error, ErrorKind};
use super::header::{encode, decode};
use super::ServicePair;

pub struct Service {
    pub api: String,
    pub msg_type: String,
    pub service: String,
}

fn header_matches<T: ServicePair>(fields: &HashMap<String, String>, service: &str) -> bool {
    if fields.get("service") != Some(&String::from(service)) && fields.get("callerid") == None {
        return false;
    }
    if fields.get("probe") == Some(&String::from("1")) {
        return true;
    }
    fields.get("md5sum") == Some(&T::md5sum())
}

fn write_response<T, U>(mut stream: &mut U, node_name: &str) -> Result<(), Error>
    where T: ServicePair,
          U: std::io::Write
{
    let mut fields = HashMap::<String, String>::new();
    fields.insert(String::from("callerid"), String::from(node_name));
    fields.insert(String::from("md5sum"), T::md5sum());
    fields.insert(String::from("type"), T::msg_type());
    encode(fields, &mut stream)
}

fn exchange_headers<T, U>(mut stream: &mut U, service: &str, node_name: &str) -> Result<(), Error>
    where T: ServicePair,
          U: std::io::Write + std::io::Read
{
    if header_matches::<T>(&decode(stream)?, service) {
        write_response::<T, U>(stream, node_name)
    } else {
        Err(ErrorKind::Mismatch.into())
    }
}

fn listen_for_clients<T, U, V, F>(service: String,
                                  node_name: String,
                                  handler: F,
                                  listener: V)
                                  -> Result<(), Error>
    where T: ServicePair,
          U: std::io::Read + std::io::Write + Send + 'static,
          V: Iterator<Item = U>,
          F: Fn(T::Request) -> T::Response + Send + Sync + 'static
{
    let handler = Arc::new(handler);
    for mut stream in listener {
        if let Err(err) = exchange_headers::<T, _>(&mut stream, &service, &node_name) {
            error!("Failed to exchange headers for service '{}': {}",
                   service,
                   err);
            continue;
        }
        let h = handler.clone();
        thread::spawn(move || respond_to::<T, U, F>(stream, h));
    }

    Ok(())
}

fn respond_to<T, U, F>(mut stream: U, handler: Arc<F>)
    where T: ServicePair,
          U: std::io::Read + std::io::Write + Send,
          F: Fn(T::Request) -> T::Response
{
    loop {
        let mut encoder = Encoder::new();
        let mut decoder = match DecoderSource::new(&mut stream).next() {
            Some(decoder) => decoder,
            None => break,
        };
        let req = match T::Request::decode(&mut decoder) {
            Ok(req) => req,
            Err(_) => break,
        };
        let res = handler(req);
        true.encode(&mut encoder).unwrap();
        res.encode(&mut encoder).unwrap();
        encoder.write_to(&mut stream).unwrap();
    }
    let mut encoder = Encoder::new();
    false.encode(&mut encoder).unwrap();
    "Failed to parse passed arguments".encode(&mut encoder).unwrap();
    encoder.write_to(&mut stream).unwrap();
}

impl Service {
    pub fn new<T, F>(hostname: &str,
                     port: u16,
                     service: &str,
                     node_name: &str,
                     handler: F)
                     -> Result<Service, Error>
        where T: ServicePair,
              F: Fn(T::Request) -> T::Response + Send + Sync + 'static
    {
        let listener = TcpListener::bind((hostname, port))?;
        let socket_address = listener.local_addr()?;
        let api = format!("rosrpc://{}:{}", hostname, socket_address.port());
        Ok(Service::wrap_stream::<T, _, _, _>(service,
                                              node_name,
                                              handler,
                                              TcpIterator::new(listener, service),
                                              &api))
    }

    fn wrap_stream<T, U, V, F>(service: &str,
                               node_name: &str,
                               handler: F,
                               listener: V,
                               api: &str)
                               -> Service
        where T: ServicePair,
              U: std::io::Read + std::io::Write + Send + 'static,
              V: Iterator<Item = U> + Send + 'static,
              F: Fn(T::Request) -> T::Response + Send + Sync + 'static
    {
        let service_name = String::from(service);
        let node_name = String::from(node_name);
        thread::spawn(move || {
            listen_for_clients::<T, _, _, _>(service_name, node_name, handler, listener)
        });
        Service {
            api: String::from(api),
            msg_type: T::msg_type(),
            service: String::from(service),
        }
    }
}

struct TcpIterator {
    listener: TcpListener,
    service: String,
}

impl TcpIterator {
    pub fn new(listener: TcpListener, service: &str) -> TcpIterator {
        TcpIterator {
            listener: listener,
            service: String::from(service),
        }
    }
}

impl Iterator for TcpIterator {
    type Item = TcpStream;

    fn next(&mut self) -> Option<Self::Item> {
        match self.listener.accept() {
            Ok((stream, _)) => Some(stream),
            Err(err) => {
                error!("TCP connection to subscriber failed on service '{}': {}",
                       self.service,
                       err);
                self.next()
            }
        }
    }
}
