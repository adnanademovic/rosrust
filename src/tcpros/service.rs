use rustc_serialize::{Encodable, Decodable};
use std::net::{TcpListener, TcpStream, ToSocketAddrs};
use std::thread;
use std::collections::HashMap;
use std;
use super::decoder::DecoderSource;
use super::encoder::Encoder;
use super::error::Error;
use super::header::{encode, decode};
use super::ServicePair;

pub struct Service {
    pub port: u16,
    pub msg_type: String,
    pub service: String,
}

fn header_matches<T: ServicePair>(fields: &HashMap<String, String>, service: &str) -> bool {
    fields.get("md5sum") == Some(&T::md5sum()) && fields.get("type") == Some(&T::msg_type()) &&
    fields.get("service") == Some(&String::from(service)) && fields.get("callerid") != None
}

fn read_request<T, U>(mut stream: &mut U, service: &str) -> Result<(), Error>
    where T: ServicePair,
          U: std::io::Read
{
    if header_matches::<T>(&decode(&mut stream)?, service) {
        Ok(())
    } else {
        Err(Error::Mismatch)
    }
}

fn write_response<U: std::io::Write>(mut stream: &mut U, node_name: &str) -> Result<(), Error> {
    let mut fields = HashMap::<String, String>::new();
    fields.insert(String::from("callerid"), String::from(node_name));
    encode(fields, &mut stream)
}

fn exchange_headers<T, U>(mut stream: &mut U, service: &str, node_name: &str) -> Result<(), Error>
    where T: ServicePair,
          U: std::io::Write + std::io::Read
{
    read_request::<T, U>(&mut stream, service)?;
    write_response::<U>(&mut stream, node_name)
}

fn listen_for_clients<T, U, V, F>(service: String,
                                  node_name: String,
                                  handler: F,
                                  listener: V)
                                  -> Result<(), Error>
    where T: ServicePair,
          U: std::io::Read + std::io::Write + Send + 'static,
          V: Iterator<Item = U>,
          F: Fn(T::Request) -> T::Response + Copy + Send + 'static
{
    for mut stream in listener {
        if let Err(err) = exchange_headers::<T, _>(&mut stream, &service, &node_name) {
            error!("Failed to exchange headers for service '{}': {}",
                   service,
                   err);
            continue;
        }
        thread::spawn(move || respond_to::<T, U, F>(stream, handler));
    }

    Ok(())
}

fn respond_to<T, U, F>(mut stream: U, handler: F)
    where T: ServicePair,
          U: std::io::Read + std::io::Write + Send,
          F: Fn(T::Request) -> T::Response + Copy + Send + 'static
{
    loop {
        let req = T::Request::decode(&mut DecoderSource::new(&mut stream).next().unwrap()).unwrap();
        let res = handler(req);
        let mut encoder = Encoder::new();
        res.encode(&mut encoder).unwrap();
        encoder.write_to(&mut stream).unwrap();
    }
}

impl Service {
    pub fn new<T, U, F>(address: U,
                        service: &str,
                        node_name: &str,
                        handler: F)
                        -> Result<Service, Error>
        where T: ServicePair,
              U: ToSocketAddrs,
              F: Fn(T::Request) -> T::Response + Copy + Send + 'static
    {
        let listener = TcpListener::bind(address)?;
        let socket_address = listener.local_addr()?;
        Ok(Service::wrap_stream::<T, _, _, _>(service,
                                              node_name,
                                              handler,
                                              TcpIterator::new(listener, service),
                                              socket_address.port()))
    }

    fn wrap_stream<T, U, V, F>(service: &str,
                               node_name: &str,
                               handler: F,
                               listener: V,
                               port: u16)
                               -> Service
        where T: ServicePair,
              U: std::io::Read + std::io::Write + Send + 'static,
              V: Iterator<Item = U> + Send + 'static,
              F: Fn(T::Request) -> T::Response + Copy + Send + 'static
    {
        let service_name = String::from(service);
        let node_name = String::from(node_name);
        thread::spawn(move || {
            listen_for_clients::<T, _, _, _>(service_name, node_name, handler, listener)
        });
        Service {
            port: port,
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
