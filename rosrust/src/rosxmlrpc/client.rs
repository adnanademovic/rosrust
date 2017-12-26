use serde::{Deserialize, Serialize};
use xml_rpc::{self, Params, Value};
use super::{Response, ResponseError, ERROR_CODE, FAILURE_CODE, SUCCESS_CODE};

pub struct Client {
    master_uri: String,
}

impl Client {
    pub fn new(master_uri: &str) -> Result<Client, xml_rpc::error::Error> {
        Ok(Client {
            master_uri: master_uri.to_owned(),
        })
    }

    pub fn request_tree_with_tree(&self, name: &str, params: Params) -> Response<Value> {
        let mut response = xml_rpc::call_value(&self.master_uri.parse().unwrap(), name, params)
            .map_err(|err| {
                ResponseError::Client(format!("Failed to perform call to server: {}", err))
            })?
            .map_err(|fault| {
                ResponseError::Client(format!(
                    "Unexpected fault #{} received from server: {}",
                    fault.code, fault.message
                ))
            })?
            .into_iter();
        let mut first_item = response.next();
        while let Some(Value::Array(v)) = first_item {
            response = v.into_iter();
            first_item = response.next();
        }
        match (first_item, response.next(), response.next()) {
            (Some(Value::Int(code)), Some(Value::String(message)), Some(data)) => match code {
                ERROR_CODE => Err(ResponseError::Client(message)),
                FAILURE_CODE => Err(ResponseError::Server(message)),
                SUCCESS_CODE => Ok(data),
                _ => Err(ResponseError::Server(
                    "Bad response code returned from server".into(),
                )),
            },
            (code, message, data) => Err(ResponseError::Server(format!(
                "Response with three parameters (int code, str msg, value) \
                 expected from server, received: ({:?}, {:?}, {:?})",
                code, message, data
            ))),
        }
    }

    pub fn request_tree<S>(&self, name: &str, params: &S) -> Response<Value>
    where
        S: Serialize,
    {
        let params = xml_rpc::into_params(params).map_err(|err| {
            ResponseError::Client(format!("Failed to serialize parameters: {}", err))
        })?;
        self.request_tree_with_tree(name, params)
    }

    pub fn request<'a, S, D>(&self, name: &str, params: &S) -> Response<D>
    where
        S: Serialize,
        D: Deserialize<'a>,
    {
        let data = self.request_tree(name, params)?;
        Deserialize::deserialize(data).map_err(|err| {
            ResponseError::Server(format!("Response data has unexpected structure: {}", err))
        })
    }
}
