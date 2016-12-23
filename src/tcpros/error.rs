use std;
use std::error::Error as ErrorTrait;

#[derive(Debug)]
pub enum Error {
    UnsupportedData,
    Mismatch,
    Io(std::io::Error),
    Other(String),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            Error::UnsupportedData | Error::Mismatch => write!(f, "Error: {}", self.description()),
            Error::Io(ref err) => write!(f, "IO error: {}", err),
            Error::Other(ref err) => write!(f, "Decoding error: {}", err),
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Error {
        Error::Io(err)
    }
}

impl std::error::Error for Error {
    fn description(&self) -> &str {
        match *self {
            Error::UnsupportedData => "Data type not supported by TCPROS",
            Error::Mismatch => "Data decoded does not match the expected structure",
            Error::Io(ref err) => err.description(),
            Error::Other(ref err) => err,
        }
    }

    fn cause(&self) -> Option<&std::error::Error> {
        match *self {
            Error::UnsupportedData | Error::Mismatch | Error::Other(..) => None,
            Error::Io(ref err) => Some(err),
        }
    }
}
