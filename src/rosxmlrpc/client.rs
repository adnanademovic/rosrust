use hyper;
use rustc_serialize::{Encodable, Decodable};
use super::serde;
use super::error;

pub struct Client {
    http_client: hyper::Client,
    server_uri: String,
}

impl Client {
    pub fn new(server_uri: &str) -> Client {
        Client {
            http_client: hyper::Client::new(),
            server_uri: server_uri.to_owned(),
        }
    }

    pub fn request<T: Decodable>(&self,
                                 function_name: &str,
                                 parameters: &[&str])
                                 -> ClientResult<T> {
        self.request_long::<T, ()>(function_name, parameters, None)
    }

    pub fn request_long<T: Decodable, Targ: Encodable>(&self,
                                                       function_name: &str,
                                                       parameters: &[&str],
                                                       extra_parameter: Option<&Targ>)
                                                       -> ClientResult<T> {
        let mut encoder = serde::Encoder::new();
        for param in parameters {
            param.encode(&mut encoder)?;
        }
        if let Some(extra_param) = extra_parameter {
            extra_param.encode(&mut encoder)?;
        }

        let mut body = Vec::<u8>::new();
        encoder.write_request(function_name, &mut body)?;

        let body = String::from_utf8(body)?;
        let res = self.http_client
            .post(&self.server_uri)
            .body(&body)
            .send()?;

        let mut res = serde::Decoder::new_response(res)?;

        Ok(T::decode(&mut res.pop()
            .ok_or(error::Error::Decoding(serde::value::DecodeError::UnsupportedDataFormat))?)?)
    }
}

pub type ClientResult<T> = Result<T, error::Error>;
