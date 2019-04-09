use super::ResponseInfo;
use std::net::SocketAddr;
use xml_rpc::{self, rouille, Value};

use super::{Response, ResponseError};

pub struct Server {
    server: xml_rpc::Server,
}

impl Default for Server {
    fn default() -> Self {
        let mut server = xml_rpc::Server::default();
        server.set_on_missing(on_missing);
        Server { server }
    }
}

impl Server {
    #[inline]
    pub fn register_value<T>(&mut self, name: impl Into<String>, msg: &'static str, handler: T)
    where
        T: Fn(xml_rpc::Params) -> Response<Value> + Send + Sync + 'static,
    {
        self.server.register_value(name, move |args| {
            let response = handler(args);
            let response_info = ResponseInfo::from_response(response, msg);
            response_info.into()
        })
    }

    #[inline]
    pub fn bind(
        self,
        uri: &SocketAddr,
    ) -> xml_rpc::error::Result<
        xml_rpc::server::BoundServer<
            impl Fn(&rouille::Request) -> rouille::Response + Send + Sync + 'static,
        >,
    > {
        self.server.bind(uri)
    }
}

#[allow(clippy::needless_pass_by_value)]
#[inline]
fn on_missing(_params: xml_rpc::Params) -> xml_rpc::Response {
    let error_message = ResponseError::Client("Bad method requested".into());
    let info = ResponseInfo::from_response_error(error_message);
    info.into()
}
