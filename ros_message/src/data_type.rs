use crate::{Error, MessagePath, Result};
use serde_derive::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use std::fmt::Formatter;

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DataType {
    Bool,
    I8(bool),
    I16,
    I32,
    I64,
    U8(bool),
    U16,
    U32,
    U64,
    F32,
    F64,
    String,
    Time,
    Duration,
    LocalStruct(String),
    RemoteStruct(MessagePath),
}

impl fmt::Display for DataType {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            DataType::Bool => BOOL_KEY.fmt(f),
            DataType::I8(true) => INT8_KEY.fmt(f),
            DataType::I8(false) => BYTE_KEY.fmt(f),
            DataType::I16 => INT16_KEY.fmt(f),
            DataType::I32 => INT32_KEY.fmt(f),
            DataType::I64 => INT64_KEY.fmt(f),
            DataType::U8(true) => UINT8_KEY.fmt(f),
            DataType::U8(false) => CHAR_KEY.fmt(f),
            DataType::U16 => UINT16_KEY.fmt(f),
            DataType::U32 => UINT32_KEY.fmt(f),
            DataType::U64 => UINT64_KEY.fmt(f),
            DataType::F32 => FLOAT32_KEY.fmt(f),
            DataType::F64 => FLOAT64_KEY.fmt(f),
            DataType::String => STRING_KEY.fmt(f),
            DataType::Time => TIME_KEY.fmt(f),
            DataType::Duration => DURATION_KEY.fmt(f),
            DataType::LocalStruct(ref name) => name.fmt(f),
            DataType::RemoteStruct(ref message) => message.fmt(f),
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
    pub fn parse(datatype: &str) -> Result<Self> {
        Ok(match datatype {
            BOOL_KEY => DataType::Bool,
            INT8_KEY => DataType::I8(true),
            BYTE_KEY => DataType::I8(false),
            INT16_KEY => DataType::I16,
            INT32_KEY => DataType::I32,
            INT64_KEY => DataType::I64,
            UINT8_KEY => DataType::U8(true),
            CHAR_KEY => DataType::U8(false),
            UINT16_KEY => DataType::U16,
            UINT32_KEY => DataType::U32,
            UINT64_KEY => DataType::U64,
            FLOAT32_KEY => DataType::F32,
            FLOAT64_KEY => DataType::F64,
            STRING_KEY => DataType::String,
            TIME_KEY => DataType::Time,
            DURATION_KEY => DataType::Duration,
            "Header" => DataType::RemoteStruct(MessagePath::new("std_msgs", "Header")?),
            _ => {
                let parts = datatype.splitn(2, '/').collect::<Vec<&str>>();
                match parts[..] {
                    [name] => DataType::LocalStruct(name.into()),
                    [package, name] => DataType::RemoteStruct(MessagePath::new(package, name)?),
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
            DataType::LocalStruct(_) | DataType::RemoteStruct(_) => false,
        }
    }

    pub fn md5_string(
        &self,
        package: &str,
        hashes: &HashMap<MessagePath, String>,
    ) -> Result<String> {
        Ok(match *self {
            DataType::Bool => BOOL_KEY,
            DataType::I8(true) => INT8_KEY,
            DataType::I8(false) => BYTE_KEY,
            DataType::I16 => INT16_KEY,
            DataType::I32 => INT32_KEY,
            DataType::I64 => INT64_KEY,
            DataType::U8(true) => UINT8_KEY,
            DataType::U8(false) => CHAR_KEY,
            DataType::U16 => UINT16_KEY,
            DataType::U32 => UINT32_KEY,
            DataType::U64 => UINT64_KEY,
            DataType::F32 => FLOAT32_KEY,
            DataType::F64 => FLOAT64_KEY,
            DataType::String => STRING_KEY,
            DataType::Time => TIME_KEY,
            DataType::Duration => DURATION_KEY,
            DataType::LocalStruct(ref name) => hashes
                .get(&MessagePath::new(package, name)?)
                .ok_or_else(|| Error::MessageDependencyMissing {
                    package: package.into(),
                    name: name.into(),
                })?
                .as_str(),
            DataType::RemoteStruct(ref message) => hashes
                .get(message)
                .ok_or_else(|| Error::MessageDependencyMissing {
                    package: message.package().into(),
                    name: message.name().into(),
                })?
                .as_str(),
        }
        .into())
    }
}
