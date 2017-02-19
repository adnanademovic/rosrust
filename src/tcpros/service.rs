use rustc_serialize::{Encodable, Decodable};
use std::net::{TcpListener, TcpStream};
use std::thread;
use std::sync::Arc;
use std::sync::mpsc::{channel, Sender, Receiver};
use std::collections::HashMap;
use std;
use super::decoder::DecoderSource;
use super::encoder::Encoder;
use super::error::{ErrorKind, Result};
use super::header::{encode, decode, match_field};
use super::{ServicePair, ServiceResult};

pub struct Service {
    pub api: String,
    pub msg_type: String,
    pub service: String,
    _raii: TcpRaii,
}

fn read_request<T: ServicePair, U: std::io::Read>(mut stream: &mut U, service: &str) -> Result<()> {
    let fields = decode(stream)?;
    match_field(&fields, "service", service)?;
    if fields.get("callerid").is_none() {
        bail!(ErrorKind::HeaderMissingField("callerid".into()));
    }
    if match_field(&fields, "probe", "1").is_ok() {
        return Ok(());
    }
    match_field(&fields, "md5sum", &T::md5sum())
}

fn write_response<T, U>(mut stream: &mut U, node_name: &str) -> Result<()>
    where T: ServicePair,
          U: std::io::Write
{
    let mut fields = HashMap::<String, String>::new();
    fields.insert(String::from("callerid"), String::from(node_name));
    fields.insert(String::from("md5sum"), T::md5sum());
    fields.insert(String::from("type"), T::msg_type());
    encode(&mut stream, &fields)?;
    Ok(())
}

fn exchange_headers<T, U>(mut stream: &mut U, service: &str, node_name: &str) -> Result<()>
    where T: ServicePair,
          U: std::io::Write + std::io::Read
{
    read_request::<T, U>(stream, service)?;
    write_response::<T, U>(stream, node_name)
}

fn listen_for_clients<T, U, V, F>(service: String, node_name: String, handler: F, listener: V)
    where T: ServicePair,
          U: std::io::Read + std::io::Write + Send + 'static,
          V: Iterator<Item = U>,
          F: Fn(T::Request) -> ServiceResult<T::Response> + Send + Sync + 'static
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
        thread::spawn(move || if let Err(err) = respond_to::<T, U, F>(stream, h) {
            let info =
                err.iter().map(|v| format!("{}", v)).collect::<Vec<_>>().join("\nCaused by:");
            error!("{}", info);
        });
    }
}

fn respond_to<T, U, F>(mut stream: U, handler: Arc<F>) -> Result<()>
    where T: ServicePair,
          U: std::io::Read + std::io::Write + Send,
          F: Fn(T::Request) -> ServiceResult<T::Response>
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
        match handler(req) {
            Ok(res) => {
                true.encode(&mut encoder)?;
                res.encode(&mut encoder)?;
            }
            Err(message) => {
                false.encode(&mut encoder)?;
                message.encode(&mut encoder)?;
            }
        }
        encoder.write_to(&mut stream)?;
    }
    let mut encoder = Encoder::new();
    false.encode(&mut encoder)?;
    "Failed to parse passed arguments".encode(&mut encoder)?;
    encoder.write_to(&mut stream)?;
    Ok(())
}

impl Service {
    pub fn new<T, F>(hostname: &str,
                     port: u16,
                     service: &str,
                     node_name: &str,
                     handler: F)
                     -> Result<Service>
        where T: ServicePair,
              F: Fn(T::Request) -> ServiceResult<T::Response> + Send + Sync + 'static
    {
        let listener = TcpListener::bind((hostname, port))?;
        let socket_address = listener.local_addr()?;
        let api = format!("rosrpc://{}:{}", hostname, socket_address.port());
        let (raii, listener) = TcpIterator::new(listener, service);
        Ok(Service::wrap_stream::<T, _, _, _>(service, node_name, handler, raii, listener, &api))
    }

    fn wrap_stream<T, U, V, F>(service: &str,
                               node_name: &str,
                               handler: F,
                               raii: TcpRaii,
                               listener: V,
                               api: &str)
                               -> Service
        where T: ServicePair,
              U: std::io::Read + std::io::Write + Send + 'static,
              V: Iterator<Item = U> + Send + 'static,
              F: Fn(T::Request) -> ServiceResult<T::Response> + Send + Sync + 'static
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
            _raii: raii,
        }
    }
}

struct TcpRaii {
    killer: Sender<Option<TcpStream>>,
}

impl Drop for TcpRaii {
    fn drop(&mut self) {
        if self.killer.send(None).is_err() {
            error!("TCP connection listener has already been killed");
        }
    }
}

struct TcpIterator {
    listener: Receiver<Option<TcpStream>>,
}

impl TcpIterator {
    pub fn new(listener: TcpListener, service: &str) -> (TcpRaii, TcpIterator) {
        let (tx, rx) = channel();
        let killer = TcpRaii { killer: tx.clone() };
        let service = String::from(service);
        thread::spawn(move || for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    if tx.send(Some(stream)).is_err() {
                        break;
                    }
                }
                Err(err) => {
                    error!("TCP connection to subscriber failed on service '{}': {}",
                           service,
                           err);
                }
            }
        });
        (killer, TcpIterator { listener: rx })
    }
}

impl Iterator for TcpIterator {
    type Item = TcpStream;

    fn next(&mut self) -> Option<Self::Item> {
        self.listener.recv().unwrap_or(None)
    }
}
