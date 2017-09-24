use std::net::SocketAddr;
use xml_rpc;

use super::{ERROR_CODE, FAILURE_CODE, SUCCESS_CODE, Response, ResponseError};

pub struct Server {
    server: xml_rpc::Server,
}

impl Default for Server {
    fn default() -> Self {
        let mut server = xml_rpc::Server::default();
        server.set_on_missing(on_missing);
        Server { server: server }
    }
}

impl Server {
    pub fn register_value<K, T>(&mut self, name: K, msg: &'static str, handler: T)
    where
        K: Into<String>,
        T: Fn(xml_rpc::Params) -> Response<xml_rpc::Value> + Send + Sync + 'static,
    {
        self.server.register_value(name, move |args| {
            Ok(response_to_params(msg, handler(args)))
        })
    }

    pub fn run(self, uri: &SocketAddr) -> xml_rpc::error::Result<()> {
        self.server.run(uri)
    }
}

fn response_to_params(msg: &str, response: Response<xml_rpc::Value>) -> xml_rpc::Params {
    use xml_rpc::Value;
    match response {
        Ok(v) => vec![Value::Int(SUCCESS_CODE), Value::String(msg.into()), v],
        Err(ResponseError::Client(err)) => {
            vec![
                Value::Int(ERROR_CODE),
                Value::String(err.into()),
                Value::Int(0),
            ]
        }
        Err(ResponseError::Server(err)) => {
            vec![
                Value::Int(FAILURE_CODE),
                Value::String(err.into()),
                Value::Int(0),
            ]
        }
    }
}

fn error_response<T>(message: T) -> xml_rpc::Params
where
    T: Into<String>,
{
    use xml_rpc::Value;
    vec![
        Value::Int(ERROR_CODE),
        Value::String(message.into()),
        Value::Int(0),
    ]
}

fn on_missing(_params: xml_rpc::Params) -> xml_rpc::Response {
    Ok(error_response("Bad method requested"))
}
