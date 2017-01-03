use hyper;
use rustc_serialize::{Encodable, Decodable};
use super::error::{self, ErrorKind, Result};
use super::serde;

pub struct Client {
    http_client: hyper::Client,
    server_uri: String,
}

impl Client {
    pub fn new(server_uri: &str) -> Client {
        Client {
            http_client: hyper::Client::new(),
            server_uri: String::from(server_uri),
        }
    }

    pub fn request_tree(&self, request: Request) -> Result<serde::XmlRpcValue> {
        let mut body = Vec::<u8>::new();
        request.encoder.write_request(&request.name, &mut body)?;

        let body = String::from_utf8(body)?;
        let res = self.http_client
            .post(&self.server_uri)
            .body(&body)
            .send()?;

        let mut res = serde::Decoder::new_response(res)?;
        match res.pop() {
            Some(v) => Ok(v.value()),
            None => {
                bail!(ErrorKind::Serde(error::serde::ErrorKind::Decoding("request tree".into())))
            }
        }
    }

    pub fn request<T: Decodable>(&self, request: Request) -> Result<T> {
        let mut body = Vec::<u8>::new();
        request.encoder.write_request(&request.name, &mut body)?;

        let body = String::from_utf8(body)?;
        let res = self.http_client
            .post(&self.server_uri)
            .body(&body)
            .send()?;

        let mut res = serde::Decoder::new_response(res)?;
        let mut value = match res.pop() {
            Some(v) => v,
            None => bail!(ErrorKind::Serde(error::serde::ErrorKind::Decoding("request".into()))),
        };
        T::decode(&mut value).map_err(|v| v.into())
    }
}

pub struct Request {
    name: String,
    encoder: serde::Encoder,
}

impl Request {
    pub fn new(function_name: &str) -> Request {
        Request {
            name: String::from(function_name),
            encoder: serde::Encoder::new(),
        }
    }

    pub fn add<T: Encodable>(&mut self, parameter: &T) -> error::serde::Result<()> {
        parameter.encode(&mut self.encoder)
    }
}
