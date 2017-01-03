use hyper;
use hyper::server::{Request, Response, Handler};
use rustc_serialize::{Decodable, Encodable};
use std;
use super::serde;
use super::error::{self, Result};

pub struct Server {
    listener: hyper::server::Listening,
    pub uri: String,
}

impl Server {
    pub fn new<T>(hostname: &str, port: u16, responder: T) -> Result<Server>
        where T: 'static + XmlRpcServer + Send + Sync
    {
        let listener = hyper::Server::http((hostname, port))?
            .handle(XmlRpcHandler::new(responder))?;
        let uri = format!("http://{}:{}/", hostname, listener.socket.port());
        Ok(Server {
            listener: listener,
            uri: uri,
        })
    }

    pub fn shutdown(&mut self) -> hyper::Result<()> {
        self.listener.close()
    }
}

pub type ParameterIterator = std::iter::Map<std::vec::IntoIter<serde::decoder::Decoder>,
                                            fn(serde::decoder::Decoder) -> Parameter>;

pub trait XmlRpcServer {
    fn handle(&self, method_name: &str, params: ParameterIterator) -> error::serde::Result<Answer>;
}

struct XmlRpcHandler<T: XmlRpcServer + Sync + Send> {
    handler: T,
}

impl<T: XmlRpcServer + Sync + Send> XmlRpcHandler<T> {
    fn new(handler: T) -> XmlRpcHandler<T> {
        XmlRpcHandler { handler: handler }
    }

    fn process(&self, req: Request, res: Response) -> Result<()> {
        let (method_name, parameters) = serde::Decoder::new_request(req)?;
        res.send(&self.handler
                .handle(&method_name, parameters.into_iter().map(Parameter::new))?
                .write_response()?)?;
        Ok(())
    }
}

impl<T: XmlRpcServer + Sync + Send> Handler for XmlRpcHandler<T> {
    fn handle(&self, req: Request, res: Response) {
        if let Err(err) = self.process(req, res) {
            let info =
                err.iter().map(|v| format!("{}", v)).collect::<Vec<_>>().join("\nCaused by:");
            error!("Server handler error: {}", info);
        }
    }
}

pub struct Answer {
    encoder: serde::Encoder,
}

impl Answer {
    pub fn new() -> Answer {
        Answer { encoder: serde::Encoder::new() }
    }

    pub fn add<T: Encodable>(&mut self, data: &T) -> error::serde::Result<()> {
        data.encode(&mut self.encoder)
    }

    fn write_response(self) -> std::io::Result<Vec<u8>> {
        let mut data = vec![];
        self.encoder.write_response(&mut data).and(Ok(data))
    }
}

pub struct Parameter {
    decoder: serde::Decoder,
}

impl Parameter {
    fn new(decoder: serde::Decoder) -> Parameter {
        Parameter { decoder: decoder }
    }

    pub fn read<T: Decodable>(mut self) -> error::serde::Result<T> {
        T::decode(&mut self.decoder)
    }

    pub fn value(self) -> serde::XmlRpcValue {
        self.decoder.value()
    }
}
