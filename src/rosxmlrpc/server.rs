extern crate hyper;
extern crate rustc_serialize;

use self::hyper::server::{Request, Response, Handler};
use super::serde;

#[allow(dead_code)]
pub struct Server {
    listener: hyper::server::Listening,
    pub uri: String,
}

impl Server {
    pub fn new<T>(server_uri: &str, responder: T) -> Result<Server, ()>
        where T: 'static + XmlRpcServer + Send + Sync
    {
        let listener = hyper::Server::http(server_uri)
            .unwrap()
            .handle(XmlRpcHandler::new(responder))
            .unwrap();
        let uri = format!("http://{}/", listener.socket);
        Ok(Server {
            listener: listener,
            uri: uri,
        })
    }
}

pub trait XmlRpcServer {
    fn handle(&self,
              method_name: &str,
              parameter_count: usize,
              req: &mut serde::Decoder,
              res: &mut serde::Encoder<&mut Vec<u8>>);
}

struct XmlRpcHandler<T: XmlRpcServer + Sync + Send> {
    handler: T,
}

impl<T: XmlRpcServer + Sync + Send> XmlRpcHandler<T> {
    fn new(handler: T) -> XmlRpcHandler<T> {
        XmlRpcHandler { handler: handler }
    }
}

impl<T: XmlRpcServer + Sync + Send> Handler for XmlRpcHandler<T> {
    fn handle(&self, req: Request, res: Response) {
        println!("WOAH");
        let mut body = Vec::<u8>::new();
        let mut request = serde::Decoder::new(req);

        {
            let mut response = serde::Encoder::new(&mut body);
            let (method_name, parameter_count) = request.peel_request_body().unwrap();
            response.start_response().unwrap();
            self.handler.handle(&method_name, parameter_count, &mut request, &mut response);
            response.end_response().unwrap();
        }

        res.send(&body).unwrap()
    }
}
