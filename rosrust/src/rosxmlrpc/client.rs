use super::error::{ErrorKind, Result, ResultExt};
use super::{Response, ResponseError, ResponseInfo};
use serde::{Deserialize, Serialize};
use xml_rpc::{self, Params, Url, Value};

pub struct Client {
    master_uri: Url,
}

impl Client {
    pub fn new(master_uri: &str) -> Result<Client> {
        let master_uri = master_uri
            .parse()
            .chain_err(|| ErrorKind::BadUri(master_uri.into()))?;
        Ok(Client { master_uri })
    }

    pub fn request_tree_with_tree(&self, name: &str, params: Params) -> Response<Value> {
        let call_result = xml_rpc::call_value(&self.master_uri, name, params);

        let server_response = call_result.map_err(|err| {
            ResponseError::Client(format!("Failed to perform call to server: {}", err))
        })?;

        let response_parameters = server_response.map_err(|fault| {
            ResponseError::Client(format!(
                "Unexpected fault #{} received from server: {}",
                fault.code, fault.message
            ))
        })?;

        let response_parameters = remove_array_wrappers(&response_parameters[..]);

        ResponseInfo::from_array(response_parameters)?.into()
    }

    pub fn request_tree<S>(&self, name: &str, params: &S) -> Response<Value>
    where
        S: Serialize,
    {
        let params = xml_rpc::into_params(params).map_err(bad_request_structure)?;
        self.request_tree_with_tree(name, params)
    }

    pub fn request<'a, S, D>(&self, name: &str, params: &S) -> Response<D>
    where
        S: Serialize,
        D: Deserialize<'a>,
    {
        let data = self.request_tree(name, params)?;
        Deserialize::deserialize(data).map_err(bad_response_structure)
    }
}

fn remove_array_wrappers(mut data: &[Value]) -> &[Value] {
    while let [Value::Array(ref children)] = data[..] {
        data = children;
    }
    data
}

fn bad_request_structure<T: ::std::fmt::Display>(err: T) -> ResponseError {
    ResponseError::Client(format!("Failed to serialize parameters: {}", err))
}

pub(crate) fn bad_response_structure<T: ::std::fmt::Display>(err: T) -> ResponseError {
    ResponseError::Server(format!("Response data has unexpected structure: {}", err))
}
