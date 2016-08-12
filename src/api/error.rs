use std;
use rosxmlrpc;

#[derive(Debug)]
pub enum ServerError {
    Deserialization(rosxmlrpc::serde::decoder::Error),
    Protocol(String),
    Critical(String),
    Serialization(rosxmlrpc::serde::encoder::Error),
    XmlRpc(rosxmlrpc::error::Error),
}

impl From<rosxmlrpc::serde::decoder::Error> for ServerError {
    fn from(err: rosxmlrpc::serde::decoder::Error) -> ServerError {
        ServerError::Deserialization(err)
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

impl std::fmt::Display for ServerError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            ServerError::Deserialization(ref err) => write!(f, "Deserialization error: {}", err),
            ServerError::Protocol(ref err) => write!(f, "Protocol error: {}", err),
            ServerError::Critical(ref err) => write!(f, "Critical error: {}", err),
            ServerError::Serialization(ref err) => write!(f, "Serialization error: {}", err),
            ServerError::XmlRpc(ref err) => write!(f, "XML RPC error: {}", err),
        }
    }
}

impl std::error::Error for ServerError {
    fn description(&self) -> &str {
        match *self {
            ServerError::Deserialization(ref err) => err.description(),
            ServerError::Protocol(ref err) => &err,
            ServerError::Critical(ref err) => &err,
            ServerError::Serialization(ref err) => err.description(),
            ServerError::XmlRpc(ref err) => err.description(),
        }
    }

    fn cause(&self) -> Option<&std::error::Error> {
        match *self {
            ServerError::Deserialization(ref err) => Some(err),
            ServerError::Protocol(..) => None,
            ServerError::Critical(..) => None,
            ServerError::Serialization(ref err) => Some(err),
            ServerError::XmlRpc(ref err) => Some(err),
        }
    }
}
