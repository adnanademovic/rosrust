use rustc_serialize::{Decodable, Encodable};
use std::net::{TcpListener, TcpStream, ToSocketAddrs};
use std::thread;
use std::collections::HashMap;
use std;
use super::decoder::DecoderSource;
use super::encoder::Encoder;
use super::error::Error;
use super::header::{encode, decode};
use super::Message;

pub struct Service {
    pub ip: String,
    pub port: u16,
    pub msg_type: String,
    pub service: String,
}

fn header_matches<T: Message>(fields: &HashMap<String, String>, service: &str) -> bool {
    fields.get("md5sum") == Some(&T::md5sum()) && fields.get("type") == Some(&T::msg_type()) &&
    fields.get("service") == Some(&String::from(service)) && fields.get("callerid") != None
}

fn read_request<T: Message, U: std::io::Read>(mut stream: &mut U,
                                              service: &str)
                                              -> Result<(), Error> {
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
    where T: Message,
          U: std::io::Write + std::io::Read
{
    read_request::<T, U>(&mut stream, service)?;
    write_response::<U>(&mut stream, node_name)
}

fn listen_for_clients<Treq, Tres, U, V, F>(service: String,
                                           node_name: String,
                                           handler: F,
                                           listener: V)
                                           -> Result<(), Error>
    where Treq: Message + Decodable,
          Tres: Message + Encodable,
          U: std::io::Read + std::io::Write + Send + 'static,
          V: Iterator<Item = U>,
          F: Fn(Treq) -> Tres + Copy + Send + 'static
{
    for mut stream in listener {
        if let Err(err) = exchange_headers::<Treq, _>(&mut stream, &service, &node_name) {
            error!("Failed to exchange headers for service '{}': {}",
                   service,
                   err);
            continue;
        }
        thread::spawn(move || respond_to::<Treq, Tres, U, F>(stream, handler));
    }

    Ok(())
}

fn respond_to<Treq, Tres, U, F>(mut stream: U, handler: F)
    where Treq: Message + Decodable,
          Tres: Message + Encodable,
          U: std::io::Read + std::io::Write + Send,
          F: Fn(Treq) -> Tres + Copy + Send + 'static
{
    loop {
        let req = Treq::decode(&mut DecoderSource::new(&mut stream).next().unwrap()).unwrap();
        let res = handler(req);
        let mut encoder = Encoder::new();
        res.encode(&mut encoder).unwrap();
        encoder.write_to(&mut stream).unwrap();
    }
}

impl Service {
    pub fn new<Treq, Tres, U, F>(address: U,
                                 service: &str,
                                 node_name: &str,
                                 handler: F)
                                 -> Result<Service, Error>
        where Treq: Message + Decodable,
              Tres: Message + Encodable,
              U: ToSocketAddrs,
              F: Fn(Treq) -> Tres + Copy + Send + 'static
    {
        let listener = TcpListener::bind(address)?;
        let socket_address = listener.local_addr()?;
        Ok(Service::wrap_stream::<Treq, Tres, _, _, _>(service,
                                                       node_name,
                                                       handler,
                                                       TcpIterator::new(listener, service),
                                                       &format!("{}", socket_address.ip()),
                                                       socket_address.port()))
    }

    fn wrap_stream<Treq, Tres, U, V, F>(service: &str,
                                        node_name: &str,
                                        handler: F,
                                        listener: V,
                                        ip: &str,
                                        port: u16)
                                        -> Service
        where Treq: Message + Decodable,
              Tres: Message + Encodable,
              U: std::io::Read + std::io::Write + Send + 'static,
              V: Iterator<Item = U> + Send + 'static,
              F: Fn(Treq) -> Tres + Copy + Send + 'static
    {
        let service_name = String::from(service);
        let node_name = String::from(node_name);
        thread::spawn(move || {
            listen_for_clients::<Treq, Tres, _, _, _>(service_name, node_name, handler, listener)
        });
        Service {
            ip: String::from(ip),
            port: port,
            msg_type: Treq::msg_type(),
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
