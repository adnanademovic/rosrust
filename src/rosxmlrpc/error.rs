use hyper;
use std;
use super::serde;

#[derive(Debug)]
pub enum Error {
    Http(hyper::error::Error),
    Io(std::io::Error),
    Utf8(std::string::FromUtf8Error),
    Serialization(serde::encoder::Error),
    Deserialization(serde::decoder::Error),
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
            Error::Io(ref err) => write!(f, "IO error: {}", err),
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
            Error::Io(ref err) => err.description(),
            Error::Utf8(ref err) => err.description(),
            Error::Serialization(ref err) => err.description(),
            Error::Deserialization(ref err) => err.description(),
        }
    }

    fn cause(&self) -> Option<&std::error::Error> {
        match *self {
            Error::Http(ref err) => Some(err),
            Error::Io(ref err) => Some(err),
            Error::Utf8(ref err) => Some(err),
            Error::Serialization(ref err) => Some(err),
            Error::Deserialization(ref err) => Some(err),
        }
    }
}
