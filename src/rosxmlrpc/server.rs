extern crate hyper;

use hyper::server::{Request, Response, Handler};
use super::serde;
use super::error::Error;

pub struct Server {
    listener: hyper::server::Listening,
    pub uri: String,
}

impl Server {
    pub fn new<T>(server_uri: &str, responder: T) -> Result<Server, Error>
        where T: 'static + XmlRpcServer + Send + Sync
    {
        let listener = try!(try!(hyper::Server::http(server_uri))
            .handle(XmlRpcHandler::new(responder)));
        let uri = format!("http://{}/", listener.socket);
        Ok(Server {
            listener: listener,
            uri: uri,
        })
    }

    pub fn shutdown(&mut self) -> Result<(), hyper::Error> {
        self.listener.close()
    }
}

pub trait XmlRpcServer {
    fn handle(&self, method_name: &str, parameter_count: usize, req: serde::Decoder) -> Vec<u8>;
}

struct XmlRpcHandler<T: XmlRpcServer + Sync + Send> {
    handler: T,
}

impl<T: XmlRpcServer + Sync + Send> XmlRpcHandler<T> {
    fn new(handler: T) -> XmlRpcHandler<T> {
        XmlRpcHandler { handler: handler }
    }

    fn process(&self, req: Request, res: Response) -> Result<(), Error> {
        let mut request = serde::Decoder::new(req);
        let (method_name, parameter_count) = try!(request.peel_request_body());

        try!(res.send(&self.handler.handle(&method_name, parameter_count, request)));
        Ok(())
    }
}

impl<T: XmlRpcServer + Sync + Send> Handler for XmlRpcHandler<T> {
    fn handle(&self, req: Request, res: Response) {
        if let Err(err) = self.process(req, res) {
            println!("Server handler error: {}", err);
        }
    }
}
