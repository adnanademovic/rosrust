use crate::{Error, MessagePath, Result};
use serde_derive::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use std::fmt::Formatter;

/// Enumerates all data types possible in a ROS message.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DataType {
    /// Represents `bool`.
    Bool,
    /// Represents `int8` or `byte`.
    ///
    /// Variants are grouped to hint at the fact that they should be treated like
    /// the same type by most code. The key exception being matching messages for
    /// validating MD5 sums and message descriptors.
    I8(I8Variant),
    /// Represents `int16`.
    I16,
    /// Represents `int32`.
    I32,
    /// Represents `int64`.
    I64,
    /// Represents `uint8` or `char`.
    ///
    /// Variants are grouped to hint at the fact that they should be treated like
    /// the same type by most code. The key exception being matching messages for
    /// validating MD5 sums and message descriptors.
    U8(U8Variant),
    /// Represents `uint16`.
    U16,
    /// Represents `uint32`.
    U32,
    /// Represents `uint64`.
    U64,
    /// Represents `float32`.
    F32,
    /// Represents `float64`.
    F64,
    /// Represents `string`.
    String,
    /// Represents `time`.
    Time,
    /// Represents `duration`.
    Duration,
    /// Represents messages from same package.
    ///
    /// When a message in package `foo` has a field of message type `foo/Bar`, it can just
    /// reference the field as `Bar`, and would put in this variant.
    LocalMessage(String),
    /// Represents messages from any package.
    ///
    /// When a message has a field of message type `foo/Bar`, it can
    /// reference the field as `foo/Bar`, and would put in this variant.
    GlobalMessage(MessagePath),
}

/// All possible names for a signed 1 byte integer in ROS messages.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum I8Variant {
    /// Represents `int8`.
    Int8,
    /// Represents `byte`.
    Byte,
}

/// All possible names for a signed 1 byte integer in ROS messages.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum U8Variant {
    /// Represents `uint8`.
    Uint8,
    /// Represents `char`.
    Char,
}

impl fmt::Display for DataType {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            DataType::Bool => BOOL_KEY.fmt(f),
            DataType::I8(I8Variant::Int8) => INT8_KEY.fmt(f),
            DataType::I8(I8Variant::Byte) => BYTE_KEY.fmt(f),
            DataType::I16 => INT16_KEY.fmt(f),
            DataType::I32 => INT32_KEY.fmt(f),
            DataType::I64 => INT64_KEY.fmt(f),
            DataType::U8(U8Variant::Uint8) => UINT8_KEY.fmt(f),
            DataType::U8(U8Variant::Char) => CHAR_KEY.fmt(f),
            DataType::U16 => UINT16_KEY.fmt(f),
            DataType::U32 => UINT32_KEY.fmt(f),
            DataType::U64 => UINT64_KEY.fmt(f),
            DataType::F32 => FLOAT32_KEY.fmt(f),
            DataType::F64 => FLOAT64_KEY.fmt(f),
            DataType::String => STRING_KEY.fmt(f),
            DataType::Time => TIME_KEY.fmt(f),
            DataType::Duration => DURATION_KEY.fmt(f),
            DataType::LocalMessage(ref name) => name.fmt(f),
            DataType::GlobalMessage(ref message) => message.fmt(f),
        }
    }
}

const BOOL_KEY: &str = "bool";
const INT8_KEY: &str = "int8";
const BYTE_KEY: &str = "byte";
const INT16_KEY: &str = "int16";
const INT32_KEY: &str = "int32";
const INT64_KEY: &str = "int64";
const UINT8_KEY: &str = "uint8";
const CHAR_KEY: &str = "char";
const UINT16_KEY: &str = "uint16";
const UINT32_KEY: &str = "uint32";
const UINT64_KEY: &str = "uint64";
const FLOAT32_KEY: &str = "float32";
const FLOAT64_KEY: &str = "float64";
const STRING_KEY: &str = "string";
const TIME_KEY: &str = "time";
const DURATION_KEY: &str = "duration";

impl DataType {
    /// Parses the data type from the type provided in a ROS msg.
    ///
    /// # Examples
    ///
    /// ```
    /// # use ros_message::{DataType, I8Variant};
    /// # use std::convert::TryInto;
    /// #
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// assert_eq!(DataType::parse("int16")?, DataType::I16);
    /// assert_eq!(DataType::parse("float64")?, DataType::F64);
    /// assert_eq!(DataType::parse("byte")?, DataType::I8(I8Variant::Byte));
    /// assert_eq!(
    ///     DataType::parse("Header")?,
    ///     DataType::GlobalMessage("std_msgs/Header".try_into()?),
    /// );
    /// assert_eq!(
    ///     DataType::parse("Position")?,
    ///     DataType::LocalMessage("Position".into()),
    /// );
    /// assert_eq!(
    ///     DataType::parse("geometry_msgs/Position")?,
    ///     DataType::GlobalMessage("geometry_msgs/Position".try_into()?),
    /// );
    /// assert!(DataType::parse("00bad_package/Name").is_err());
    /// assert!(DataType::parse("a/bad/type").is_err());
    /// # Ok(())
    /// # }
    /// ```
    pub fn parse(datatype: &str) -> Result<Self> {
        Ok(match datatype {
            BOOL_KEY => DataType::Bool,
            INT8_KEY => DataType::I8(I8Variant::Int8),
            BYTE_KEY => DataType::I8(I8Variant::Byte),
            INT16_KEY => DataType::I16,
            INT32_KEY => DataType::I32,
            INT64_KEY => DataType::I64,
            UINT8_KEY => DataType::U8(U8Variant::Uint8),
            CHAR_KEY => DataType::U8(U8Variant::Char),
            UINT16_KEY => DataType::U16,
            UINT32_KEY => DataType::U32,
            UINT64_KEY => DataType::U64,
            FLOAT32_KEY => DataType::F32,
            FLOAT64_KEY => DataType::F64,
            STRING_KEY => DataType::String,
            TIME_KEY => DataType::Time,
            DURATION_KEY => DataType::Duration,
            "Header" => DataType::GlobalMessage(MessagePath::new("std_msgs", "Header")?),
            _ => {
                let parts = datatype.splitn(3, '/').collect::<Vec<&str>>();
                match parts[..] {
                    [name] => DataType::LocalMessage(name.into()),
                    [package, name] => DataType::GlobalMessage(MessagePath::new(package, name)?),
                    _ => {
                        return Err(Error::UnsupportedDataType {
                            name: datatype.into(),
                            reason: "string needs to be in `name` or `package/name` format".into(),
                        })
                    }
                }
            }
        })
    }

    /// Returns true if the type is a built in type, rather than another message.
    ///
    /// # Examples
    ///
    /// ```
    /// # use ros_message::{DataType, I8Variant};
    /// #
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// assert!(DataType::parse("int16")?.is_builtin());
    /// assert!(DataType::parse("float64")?.is_builtin());
    /// assert!(DataType::parse("byte")?.is_builtin());
    /// assert!(!DataType::parse("Header")?.is_builtin());
    /// assert!(!DataType::parse("Position")?.is_builtin());
    /// assert!(!DataType::parse("geometry_msgs/Position")?.is_builtin());
    /// # Ok(())
    /// # }
    /// ```
    pub fn is_builtin(&self) -> bool {
        match *self {
            DataType::Bool
            | DataType::I8(_)
            | DataType::I16
            | DataType::I32
            | DataType::I64
            | DataType::U8(_)
            | DataType::U16
            | DataType::U32
            | DataType::U64
            | DataType::F32
            | DataType::F64
            | DataType::String
            | DataType::Time
            | DataType::Duration => true,
            DataType::LocalMessage(_) | DataType::GlobalMessage(_) => false,
        }
    }

    /// Returns the representation of the data type when constructing the MD5 sum.
    ///
    /// For built in types, it is the same as the data type name.
    ///
    /// For message types, it is that message's MD5 sum, which is passed in via the `hashes`
    /// argument.
    ///
    /// The `package` argument should be the package that the current message is in, to resolve
    /// global paths of local message dependencies.
    ///
    /// # Errors
    ///
    /// An error will be returned if a message we depend upon is missing.
    ///
    /// # Examples
    ///
    /// ```
    /// # use ros_message::{DataType, I8Variant};
    /// # use std::convert::TryInto;
    /// # use std::collections::HashMap;
    /// #
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let mut hashes = HashMap::new();
    /// hashes.insert("foo/Header".try_into()?, "wrong_header".into());
    /// hashes.insert("std_msgs/Header".try_into()?, "123".into());
    /// hashes.insert("geometry_msgs/Position".try_into()?, "345".into());
    /// hashes.insert("foo/Position".try_into()?, "678".into());
    ///
    /// assert_eq!(DataType::parse("int16")?.md5_str("foo", &hashes)?, "int16");
    /// assert_eq!(DataType::parse("float64")?.md5_str("foo", &hashes)?, "float64");
    /// assert_eq!(DataType::parse("byte")?.md5_str("foo", &hashes)?, "byte");
    /// assert_eq!(DataType::parse("Header")?.md5_str("foo", &hashes)?, "123");
    /// assert_eq!(DataType::parse("Position")?.md5_str("foo", &hashes)?, "678");
    /// assert_eq!(DataType::parse("geometry_msgs/Position")?.md5_str("foo", &hashes)?, "345");
    /// assert!(DataType::parse("other_msgs/Position")?.md5_str("foo", &hashes).is_err());
    /// # Ok(())
    /// # }
    /// ```
    pub fn md5_str<'a>(
        &self,
        package: &str,
        hashes: &'a HashMap<MessagePath, String>,
    ) -> Result<&'a str> {
        Ok(match *self {
            DataType::Bool => BOOL_KEY,
            DataType::I8(I8Variant::Int8) => INT8_KEY,
            DataType::I8(I8Variant::Byte) => BYTE_KEY,
            DataType::I16 => INT16_KEY,
            DataType::I32 => INT32_KEY,
            DataType::I64 => INT64_KEY,
            DataType::U8(U8Variant::Uint8) => UINT8_KEY,
            DataType::U8(U8Variant::Char) => CHAR_KEY,
            DataType::U16 => UINT16_KEY,
            DataType::U32 => UINT32_KEY,
            DataType::U64 => UINT64_KEY,
            DataType::F32 => FLOAT32_KEY,
            DataType::F64 => FLOAT64_KEY,
            DataType::String => STRING_KEY,
            DataType::Time => TIME_KEY,
            DataType::Duration => DURATION_KEY,
            DataType::LocalMessage(ref name) => hashes
                .get(&MessagePath::new(package, name)?)
                .ok_or_else(|| Error::MessageDependencyMissing {
                    package: package.into(),
                    name: name.into(),
                })?
                .as_str(),
            DataType::GlobalMessage(ref message) => hashes
                .get(message)
                .ok_or_else(|| Error::MessageDependencyMissing {
                    package: message.package().into(),
                    name: message.name().into(),
                })?
                .as_str(),
        })
    }
}
