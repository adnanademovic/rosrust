use super::{Response, ResponseError, ERROR_CODE, FAILURE_CODE, SUCCESS_CODE};
use xml_rpc::{self, Value};

pub struct ResponseInfo {
    pub code: i32,
    pub message: String,
    pub data: Value,
}

impl ResponseInfo {
    #[inline]
    pub fn new(code: i32, message: String, data: Value) -> Self {
        Self {
            code,
            message,
            data,
        }
    }

    #[inline]
    pub fn from_array(parameters: &[Value]) -> Response<Self> {
        match *parameters {
            [Value::Int(code), Value::String(ref message), ref data] => Ok(Self::new(code, message.clone(), data.clone())),
            _ => Err(ResponseError::Server(format!(
                "Response with three parameters (int code, str msg, value) expected from server, received: {:?}",
                parameters
            ))),
        }
    }

    #[inline]
    pub fn from_response(response: Response<Value>, message: &str) -> Self {
        match response {
            Ok(data) => Self::from_response_success(data, message),
            Err(err) => Self::from_response_error(err),
        }
    }

    #[inline]
    pub fn from_response_error(err: ResponseError) -> Self {
        match err {
            ResponseError::Client(msg) => Self::from_client_error(msg),
            ResponseError::Server(msg) => Self::from_server_error(msg),
        }
    }

    #[inline]
    pub fn from_client_error(message: String) -> Self {
        Self::new(ERROR_CODE, message, Value::Int(0))
    }

    #[inline]
    pub fn from_server_error(message: String) -> Self {
        Self::new(FAILURE_CODE, message, Value::Int(0))
    }

    #[inline]
    pub fn from_response_success(data: Value, message: &str) -> Self {
        Self::new(SUCCESS_CODE, message.to_owned(), data)
    }
}

impl From<ResponseInfo> for xml_rpc::Response {
    fn from(src: ResponseInfo) -> xml_rpc::Response {
        let code = Value::Int(src.code);
        let message = Value::String(src.message);
        Ok(vec![Value::Array(vec![code, message, src.data])])
    }
}

impl From<ResponseInfo> for Response<Value> {
    fn from(src: ResponseInfo) -> Response<Value> {
        match src.code {
            SUCCESS_CODE => Ok(src.data),
            ERROR_CODE => Err(ResponseError::Client(src.message)),
            FAILURE_CODE => Err(ResponseError::Server(src.message)),
            _ => Err(ResponseError::Server(format!(
                "Bad response code \"{}\" returned from server",
                src.code
            ))),
        }
    }
}
