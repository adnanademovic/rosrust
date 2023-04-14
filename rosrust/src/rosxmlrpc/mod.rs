pub use self::client::Client;
use self::response_info::ResponseInfo;
pub use self::server::Server;

pub mod client;
pub mod error;
mod response_info;
pub mod server;

pub type Response<T> = Result<T, ResponseError>;

#[derive(Clone, Debug)]
pub enum ResponseError {
    Client(String),
    Server(String),
}

impl std::fmt::Display for ResponseError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        match *self {
            ResponseError::Client(ref v) => write!(f, "Client error: {}", v),
            ResponseError::Server(ref v) => write!(f, "Server error: {}", v),
        }
    }
}

impl std::error::Error for ResponseError {
    fn description(&self) -> &str {
        match *self {
            ResponseError::Client(ref v) | ResponseError::Server(ref v) => v,
        }
    }

    fn cause(&self) -> Option<&dyn std::error::Error> {
        None
    }
}

const ERROR_CODE: i32 = -1;
const FAILURE_CODE: i32 = 0;
const SUCCESS_CODE: i32 = 1;
