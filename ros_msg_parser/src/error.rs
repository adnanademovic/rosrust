#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("message path `{name}` is invalid, {reason}")]
    InvalidMessagePath { name: String, reason: String },
}

pub type Result<T> = std::result::Result<T, Error>;
