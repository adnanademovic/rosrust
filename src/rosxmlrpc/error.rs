use hyper;
use std;
use super::serde;

#[derive(Debug)]
pub enum Error {
    Http(hyper::error::Error),
    Io(std::io::Error),
    Utf8(std::string::FromUtf8Error),
    Serde(serde::Error),
    Decoding(serde::value::DecodeError),
}

impl From<hyper::error::Error> for Error {
    fn from(err: hyper::error::Error) -> Error {
        Error::Http(err)
    }
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Error {
        Error::Io(err)
    }
}

impl From<std::string::FromUtf8Error> for Error {
    fn from(err: std::string::FromUtf8Error) -> Error {
        Error::Utf8(err)
    }
}

impl From<serde::Error> for Error {
    fn from(err: serde::Error) -> Error {
        Error::Serde(err)
    }
}

impl From<serde::value::DecodeError> for Error {
    fn from(err: serde::value::DecodeError) -> Error {
        Error::Decoding(err)
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            Error::Http(ref err) => write!(f, "HTTP error: {}", err),
            Error::Io(ref err) => write!(f, "IO error: {}", err),
            Error::Utf8(ref err) => write!(f, "UTF8 error: {}", err),
            Error::Serde(ref err) => write!(f, "Serialization error: {}", err),
            Error::Decoding(ref err) => write!(f, "Decoding error: {}", err),
        }
    }
}

impl std::error::Error for Error {
    fn description(&self) -> &str {
        match *self {
            Error::Http(ref err) => err.description(),
            Error::Io(ref err) => err.description(),
            Error::Utf8(ref err) => err.description(),
            Error::Serde(ref err) => err.description(),
            Error::Decoding(ref err) => err.description(),
        }
    }

    fn cause(&self) -> Option<&std::error::Error> {
        match *self {
            Error::Http(ref err) => Some(err),
            Error::Io(ref err) => Some(err),
            Error::Utf8(ref err) => Some(err),
            Error::Serde(ref err) => Some(err),
            Error::Decoding(ref err) => Some(err),
        }
    }
}
