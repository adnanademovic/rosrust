/// Enumeration of all errors that can be returned.
#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// Message doesn't have a valid format.
    ///
    /// Message names must follow the `package_name/MessageName` format.
    ///
    /// Packages must follow [REP 144](https://www.ros.org/reps/rep-0144.html) rules.
    #[error("message path `{name}` is invalid, {reason}")]
    InvalidMessagePath {
        /// Full name of the message we are trying to parse.
        name: String,
        /// Reason for the failure.
        reason: String,
    },
    /// Field in the `msg` or `srv` file has a name that doesn't fit any data type category.
    #[error("data type `{name}` is invalid, {reason}")]
    UnsupportedDataType {
        /// Full name of the data type we are trying to parse.
        name: String,
        /// Reason for the failure.
        reason: String,
    },
    /// The `msg` or `srv` file being parsed has invalid content.
    #[error("bad content in message: `{0}`")]
    BadMessageContent(String),
    /// Certain operations on a `msg` or `srv` file require first handling all messages it depends upon.
    ///
    /// For example, to calculate an MD5 sum for a message, you first need to calculate it for
    /// all messages it depends upon, and passing them into the calculation call.
    #[error("message dependency missing: {package}/{name}")]
    MessageDependencyMissing {
        /// Package that the message is located in.
        package: String,
        /// Name of the missing message.
        name: String,
    },
    /// Passed in constant value is not parsable as its data type.
    #[error("bad constant value `{value}` in field {name}")]
    BadConstant {
        /// Name of the constant.
        name: String,
        /// The invalid value provided.
        value: String,
    },
}

/// Convenience type for shorter return value syntax of this crate's errors.
pub type Result<T> = std::result::Result<T, Error>;
