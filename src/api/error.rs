use nix;
use std;
use rosxmlrpc;
use tcpros;
use super::naming::Error as NamingError;
use super::master::{MasterError, FailureType};

#[derive(Debug)]
pub enum ServerError {
    XmlRpcSerde(rosxmlrpc::serde::Error),
    Protocol(String),
    Critical(String),
    XmlRpc(rosxmlrpc::error::Error),
    Tcpros(tcpros::Error),
    Io(std::io::Error),
    Nix(nix::Error),
    FromUTF8(std::string::FromUtf8Error),
    ApiFail(FailureType, String),
    Naming(NamingError),
}

impl From<rosxmlrpc::serde::Error> for ServerError {
    fn from(err: rosxmlrpc::serde::Error) -> ServerError {
        ServerError::XmlRpcSerde(err)
    }
}

impl From<rosxmlrpc::error::Error> for ServerError {
    fn from(err: rosxmlrpc::error::Error) -> ServerError {
        ServerError::XmlRpc(err)
    }
}

impl From<tcpros::Error> for ServerError {
    fn from(err: tcpros::Error) -> ServerError {
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

impl From<MasterError> for ServerError {
    fn from(err: MasterError) -> ServerError {
        match err {
            MasterError::XmlRpc(v) => ServerError::from(v),
            MasterError::ApiError(t, m) => ServerError::ApiFail(t, m),
        }
    }
}

impl From<NamingError> for ServerError {
    fn from(err: NamingError) -> ServerError {
        ServerError::Naming(err)
    }
}

impl std::fmt::Display for ServerError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            ServerError::Protocol(ref err) => write!(f, "Protocol error: {}", err),
            ServerError::Critical(ref err) => write!(f, "Critical error: {}", err),
            ServerError::XmlRpcSerde(ref err) => write!(f, "Serialization error: {}", err),
            ServerError::XmlRpc(ref err) => write!(f, "XML RPC error: {}", err),
            ServerError::Tcpros(ref err) => write!(f, "TCPROS error: {}", err),
            ServerError::Io(ref err) => write!(f, "IO error: {}", err),
            ServerError::Nix(ref err) => write!(f, "NIX error: {}", err),
            ServerError::FromUTF8(ref err) => write!(f, "From UTF-8 error: {}", err),
            ServerError::ApiFail(ref t, ref m) => write!(f, "{} in Master API: {}", t, m),
            ServerError::Naming(ref err) => write!(f, "Naming error: {}", err),
        }
    }
}

impl std::error::Error for ServerError {
    fn description(&self) -> &str {
        match *self {
            ServerError::Protocol(ref err) => &err,
            ServerError::Critical(ref err) => &err,
            ServerError::XmlRpcSerde(ref err) => err.description(),
            ServerError::XmlRpc(ref err) => err.description(),
            ServerError::Tcpros(ref err) => err.description(),
            ServerError::Io(ref err) => err.description(),
            ServerError::Nix(ref err) => err.description(),
            ServerError::FromUTF8(ref err) => err.description(),
            ServerError::ApiFail(.., ref m) => m,
            ServerError::Naming(ref err) => err.description(),
        }
    }

    fn cause(&self) -> Option<&std::error::Error> {
        match *self {
            ServerError::Protocol(..) => None,
            ServerError::Critical(..) => None,
            ServerError::XmlRpcSerde(ref err) => Some(err),
            ServerError::XmlRpc(ref err) => Some(err),
            ServerError::Tcpros(ref err) => Some(err),
            ServerError::Io(ref err) => Some(err),
            ServerError::Nix(ref err) => Some(err),
            ServerError::FromUTF8(ref err) => Some(err),
            ServerError::ApiFail(..) => None,
            ServerError::Naming(ref err) => Some(err),
        }
    }
}
