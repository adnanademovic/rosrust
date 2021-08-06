#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("message path `{name}` is invalid, {reason}")]
    InvalidMessagePath { name: String, reason: String },
    #[error("data type `{name}` is invalid, {reason}")]
    UnsupportedDataType { name: String, reason: String },
    #[error("bad content in message: `{0}`")]
    BadMessageContent(String),
    #[error("message dependency missing: {package}/{name}")]
    MessageDependencyMissing { package: String, name: String },
    #[error("bad constant value `{value}` in field {name}")]
    BadConstant { name: String, value: String },
}

pub type Result<T> = std::result::Result<T, Error>;
