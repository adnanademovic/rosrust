use std::io;

pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// Message not found in provided directories.
    #[error("message {msg} not found in provided directories\nDirectories:\n{folders}")]
    MessageNotFound { msg: String, folders: String },
    /// Message map does not contain all needed elements.
    #[error("message map does not contain all needed elements")]
    MessageMapIncomplete,
    /// Failed to read file to string.
    #[error("failed to read file to string")]
    ReadFile(#[source] io::Error),
    /// Failed to build service messages.
    #[error("failed to build service messages")]
    BuildMessage(#[source] ros_message::Error),
    /// Failed to parse all message paths.
    #[error("failed to parse all message paths")]
    ParseMessagePaths(#[source] ros_message::Error),
    /// Invalid message path.
    #[error("invalid message path")]
    MessagePath(#[source] ros_message::Error),
    /// Failed to parse message.
    #[error("failed to parse message")]
    ParseMessage(#[source] ros_message::Error),
}
