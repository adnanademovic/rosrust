use nix;
use std;
use rosxmlrpc;
use tcpros;

#[derive(Debug)]
pub enum ServerError {
    Deserialization(rosxmlrpc::serde::value::DecodeError),
    Decoding(rosxmlrpc::serde::decoder::Error),
    Protocol(String),
    Critical(String),
    Serialization(rosxmlrpc::serde::encoder::Error),
    XmlRpc(rosxmlrpc::error::Error),
    Tcpros(tcpros::error::Error),
    Io(std::io::Error),
    Nix(nix::Error),
    FromUTF8(std::string::FromUtf8Error),
}

impl From<rosxmlrpc::serde::value::DecodeError> for ServerError {
    fn from(err: rosxmlrpc::serde::value::DecodeError) -> ServerError {
        ServerError::Deserialization(err)
    }
}

impl From<rosxmlrpc::serde::decoder::Error> for ServerError {
    fn from(err: rosxmlrpc::serde::decoder::Error) -> ServerError {
        ServerError::Decoding(err)
    }
}

impl From<rosxmlrpc::serde::encoder::Error> for ServerError {
    fn from(err: rosxmlrpc::serde::encoder::Error) -> ServerError {
        ServerError::Serialization(err)
    }
}

impl From<rosxmlrpc::error::Error> for ServerError {
    fn from(err: rosxmlrpc::error::Error) -> ServerError {
        ServerError::XmlRpc(err)
    }
}

impl From<tcpros::error::Error> for ServerError {
    fn from(err: tcpros::error::Error) -> ServerError {
        ServerError::Tcpros(err)
    }
}

impl From<std::io::Error> for ServerError {
    fn from(err: std::io::Error) -> ServerError {
        ServerError::Io(err)
    }
}

impl From<nix::Error> for ServerError {
    fn from(err: nix::Error) -> ServerError {
        ServerError::Nix(err)
    }
}

impl From<std::string::FromUtf8Error> for ServerError {
    fn from(err: std::string::FromUtf8Error) -> ServerError {
        ServerError::FromUTF8(err)
    }
}

impl std::fmt::Display for ServerError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            ServerError::Deserialization(ref err) => write!(f, "Deserialization error: {}", err),
            ServerError::Decoding(ref err) => write!(f, "Decoding error: {}", err),
            ServerError::Protocol(ref err) => write!(f, "Protocol error: {}", err),
            ServerError::Critical(ref err) => write!(f, "Critical error: {}", err),
            ServerError::Serialization(ref err) => write!(f, "Serialization error: {}", err),
            ServerError::XmlRpc(ref err) => write!(f, "XML RPC error: {}", err),
            ServerError::Tcpros(ref err) => write!(f, "TCPROS error: {}", err),
            ServerError::Io(ref err) => write!(f, "IO error: {}", err),
            ServerError::Nix(ref err) => write!(f, "NIX error: {}", err),
            ServerError::FromUTF8(ref err) => write!(f, "From UTF-8 error: {}", err),
        }
    }
}

impl std::error::Error for ServerError {
    fn description(&self) -> &str {
        match *self {
            ServerError::Deserialization(ref err) => err.description(),
            ServerError::Decoding(ref err) => err.description(),
            ServerError::Protocol(ref err) => &err,
            ServerError::Critical(ref err) => &err,
            ServerError::Serialization(ref err) => err.description(),
            ServerError::XmlRpc(ref err) => err.description(),
            ServerError::Tcpros(ref err) => err.description(),
            ServerError::Io(ref err) => err.description(),
            ServerError::Nix(ref err) => err.description(),
            ServerError::FromUTF8(ref err) => err.description(),
        }
    }

    fn cause(&self) -> Option<&std::error::Error> {
        match *self {
            ServerError::Deserialization(ref err) => Some(err),
            ServerError::Decoding(ref err) => Some(err),
            ServerError::Protocol(..) => None,
            ServerError::Critical(..) => None,
            ServerError::Serialization(ref err) => Some(err),
            ServerError::XmlRpc(ref err) => Some(err),
            ServerError::Tcpros(ref err) => Some(err),
            ServerError::Io(ref err) => Some(err),
            ServerError::Nix(ref err) => Some(err),
            ServerError::FromUTF8(ref err) => Some(err),
        }
    }
}
