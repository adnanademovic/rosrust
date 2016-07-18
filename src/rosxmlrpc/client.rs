extern crate hyper;
extern crate rustc_serialize;
extern crate xml;

use self::rustc_serialize::Encodable;
use self::rustc_serialize::Decodable;
use std;
use super::serde;

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
        let mut body = Vec::<u8>::new();
        {
            let mut encoder = serde::Encoder::new(&mut body);
            try!(encoder.start_request(function_name));
            for param in parameters {
                try!(param.encode(&mut encoder));
            }
            if let Some(extra_param) = extra_parameter {
                try!(extra_param.encode(&mut encoder));
            }
            try!(encoder.end_request());
        }

        let body = try!(String::from_utf8(body));
        let res = try!(self.http_client
            .post(&self.server_uri)
            .body(&body)
            .send());

        let mut res = serde::Decoder::new(res);
        try!(res.peel_response_body());

        Ok(try!(T::decode(&mut res)))
    }
}

pub type ClientResult<T> = Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    Http(hyper::error::Error),
    Utf8(std::string::FromUtf8Error),
    Serialization(serde::encoder::Error),
    Deserialization(serde::decoder::Error),
}

impl From<hyper::error::Error> for Error {
    fn from(err: hyper::error::Error) -> Error {
        Error::Http(err)
    }
}

impl From<std::string::FromUtf8Error> for Error {
    fn from(err: std::string::FromUtf8Error) -> Error {
        Error::Utf8(err)
    }
}

impl From<serde::encoder::Error> for Error {
    fn from(err: serde::encoder::Error) -> Error {
        Error::Serialization(err)
    }
}

impl From<serde::decoder::Error> for Error {
    fn from(err: serde::decoder::Error) -> Error {
        Error::Deserialization(err)
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            Error::Http(ref err) => write!(f, "HTTP error: {}", err),
            Error::Utf8(ref err) => write!(f, "UTF8 error: {}", err),
            Error::Serialization(ref err) => write!(f, "Serialization error: {}", err),
            Error::Deserialization(ref err) => write!(f, "Deserialization error: {}", err),
        }
    }
}

impl std::error::Error for Error {
    fn description(&self) -> &str {
        match *self {
            Error::Http(ref err) => err.description(),
            Error::Utf8(ref err) => err.description(),
            Error::Serialization(ref err) => err.description(),
            Error::Deserialization(ref err) => err.description(),
        }
    }

    fn cause(&self) -> Option<&std::error::Error> {
        match *self {
            Error::Http(ref err) => Some(err),
            Error::Utf8(ref err) => Some(err),
            Error::Serialization(ref err) => Some(err),
            Error::Deserialization(ref err) => Some(err),
        }
    }
}
