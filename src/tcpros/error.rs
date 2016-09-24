use std;

#[derive(Debug)]
pub enum Error {
    UnsupportedData,
    Mismatch,
    Truncated,
    Io(std::io::Error),
    Other(String),
}

impl From<String> for Error {
    fn from(err: String) -> Error {
        Error::Other(err)
    }
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Error {
        Error::Io(err)
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            Error::UnsupportedData => write!(f, "Data type not supported by TCPROS"),
            Error::Mismatch => write!(f, "Data doesn't match the structure we're parsing"),
            Error::Truncated => write!(f, "Abrupt end of input data"),
            Error::Io(ref err) => write!(f, "IO error within TCPROS: {}", err),
            Error::Other(ref err) => write!(f, "TCPROS Decoding error: {}", err),
        }
    }
}

impl std::error::Error for Error {
    fn description(&self) -> &str {
        match *self {
            Error::UnsupportedData => "Data type not supported by TCPROS",
            Error::Mismatch => "Data doesn't match the structure we're parsing",
            Error::Truncated => "Abrupt end of input data",
            Error::Io(ref err) => err.description(),
            Error::Other(ref err) => &err,
        }
    }

    fn cause(&self) -> Option<&std::error::Error> {
        match *self {
            Error::UnsupportedData => None,
            Error::Mismatch => None,
            Error::Truncated => None,
            Error::Io(ref err) => Some(err),
            Error::Other(_) => None,
        }
    }
}
