use byteorder::WriteBytesExt;
use std::net::TcpListener;
use std::thread;
use std::sync::Arc;
use std::collections::HashMap;
use std;
use serde_rosmsg::{from_reader, to_writer};
use super::error::{ErrorKind, Result};
use super::header;
use super::util::tcpconnection;
use super::{ServicePair, ServiceResult};

pub struct Service {
    pub api: String,
    pub msg_type: String,
    pub service: String,
    _raii: tcpconnection::Raii,
}

impl Service {
    pub fn new<T, F>(
        hostname: &str,
        port: u16,
        service: &str,
        node_name: &str,
        handler: F,
    ) -> Result<Service>
    where
        T: ServicePair,
        F: Fn(T::Request) -> ServiceResult<T::Response> + Send + Sync + 'static,
    {
        let listener = TcpListener::bind((hostname, port))?;
        let socket_address = listener.local_addr()?;
        let api = format!("rosrpc://{}:{}", hostname, socket_address.port());
        let (raii, listener) = tcpconnection::iterate(listener, format!("service '{}'", service));
        Ok(Service::wrap_stream::<T, _, _, _>(
            service,
            node_name,
            handler,
            raii,
            listener,
            &api,
        ))
    }

    fn wrap_stream<T, U, V, F>(
        service: &str,
        node_name: &str,
        handler: F,
        raii: tcpconnection::Raii,
        listener: V,
        api: &str,
    ) -> Service
    where
        T: ServicePair,
        U: std::io::Read + std::io::Write + Send + 'static,
        V: Iterator<Item = U> + Send + 'static,
        F: Fn(T::Request) -> ServiceResult<T::Response> + Send + Sync + 'static,
    {
        let service_name = String::from(service);
        let node_name = String::from(node_name);
        thread::spawn(move || {
            listen_for_clients::<T, _, _, _>(&service_name, &node_name, handler, listener)
        });
        Service {
            api: String::from(api),
            msg_type: T::msg_type(),
            service: String::from(service),
            _raii: raii,
        }
    }
}

fn listen_for_clients<T, U, V, F>(service: &str, node_name: &str, handler: F, connections: V)
where
    T: ServicePair,
    U: std::io::Read + std::io::Write + Send + 'static,
    V: Iterator<Item = U>,
    F: Fn(T::Request) -> ServiceResult<T::Response> + Send + Sync + 'static,
{
    let handler = Arc::new(handler);
    for mut stream in connections {
        // Service request starts by exchanging connection headers
        if let Err(err) = exchange_headers::<T, _>(&mut stream, service, node_name) {
            error!(
                "Failed to exchange headers for service '{}': {}",
                service,
                err
            );
            continue;
        }

        // Spawn a thread for handling requests
        spawn_request_handler::<T, U, F>(stream, Arc::clone(&handler));
    }
}

fn exchange_headers<T, U>(stream: &mut U, service: &str, node_name: &str) -> Result<()>
where
    T: ServicePair,
    U: std::io::Write + std::io::Read,
{
    read_request::<T, U>(stream, service)?;
    write_response::<T, U>(stream, node_name)
}

fn read_request<T: ServicePair, U: std::io::Read>(stream: &mut U, service: &str) -> Result<()> {
    let fields = header::decode(stream)?;
    header::match_field(&fields, "service", service)?;
    if fields.get("callerid").is_none() {
        bail!(ErrorKind::HeaderMissingField("callerid".into()));
    }
    if header::match_field(&fields, "probe", "1").is_ok() {
        return Ok(());
    }
    header::match_field(&fields, "md5sum", &T::md5sum())
}

fn write_response<T, U>(stream: &mut U, node_name: &str) -> Result<()>
where
    T: ServicePair,
    U: std::io::Write,
{
    let mut fields = HashMap::<String, String>::new();
    fields.insert(String::from("callerid"), String::from(node_name));
    fields.insert(String::from("md5sum"), T::md5sum());
    fields.insert(String::from("type"), T::msg_type());
    header::encode(stream, &fields)?;
    Ok(())
}

fn spawn_request_handler<T, U, F>(stream: U, handler: Arc<F>)
where
    T: ServicePair,
    U: std::io::Read + std::io::Write + Send + 'static,
    F: Fn(T::Request) -> ServiceResult<T::Response> + Send + Sync + 'static,
{
    thread::spawn(move || {
        if let Err(err) = handle_request_loop::<T, U, F>(stream, &handler) {
            if !err.is_closed_connection() {
                let info = err.iter()
                    .map(|v| format!("{}", v))
                    .collect::<Vec<_>>()
                    .join("\nCaused by:");
                error!("{}", info);
            }
        }
    });
}

fn handle_request_loop<T, U, F>(mut stream: U, handler: &Arc<F>) -> Result<()>
where
    T: ServicePair,
    U: std::io::Read + std::io::Write,
    F: Fn(T::Request) -> ServiceResult<T::Response>,
{
    // Receive request from client
    // Break out of loop in case of failure to read request
    while let Ok(req) = from_reader(&mut stream) {
        // Call function that handles request and returns response
        match handler(req) {
            Ok(res) => {
                // Send True flag and response in case of success
                stream.write_u8(1)?;
                to_writer(&mut stream, &res)?;
            }
            Err(message) => {
                // Send False flag and error message string in case of failure
                stream.write_u8(0)?;
                to_writer(&mut stream, &message)?;
            }
        };
    }

    // Upon failure to read request, send client failure message
    // This can be caused by actual issues or by the client stopping the connection
    stream.write_u8(0)?;
    to_writer(&mut stream, &"Failed to parse passed arguments")?;
    Ok(())
}
