use crate::error::{Result, ResultExt};
use crate::{Duration, RosMsg, Time};
use lazy_static::lazy_static;
use regex::RegexBuilder;
use ros_message::{DataType, FieldCase, FieldInfo, MessagePath, MessageValue, Msg, Value};
use std::collections::HashMap;
use std::convert::TryInto;
use std::io;

#[derive(Clone, Debug)]
pub struct DynamicMsg {
    msg: Msg,
    dependencies: HashMap<MessagePath, Msg>,
}

fn get_field<'a>(value: &'a MessageValue, name: &str) -> io::Result<&'a Value> {
    value.get(name).ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::Other,
            format!("Missing field `{}` in value", name),
        )
    })
}

impl DynamicMsg {
    pub fn new(message_type: &str, message_definition: &str) -> Result<Self> {
        lazy_static! {
            static ref RE_DESCRIPTOR_MESSAGES_SPLITTER: regex::Regex = RegexBuilder::new("^=+$")
                .multi_line(true)
                .build()
                .expect("Invalid regex `^=+$`");
        }
        let mut message_bodies = RE_DESCRIPTOR_MESSAGES_SPLITTER.split(message_definition);
        let message_src = message_bodies.next().chain_err(|| {
            format!(
                "Message definition for {} is missing main message body",
                message_type,
            )
        })?;
        let msg = Self::parse_msg(message_type, message_src)?;
        let mut dependencies = HashMap::new();
        for message_body in message_bodies {
            let dependency = Self::parse_dependency(message_body)?;
            dependencies.insert(dependency.path().clone(), dependency);
        }

        Ok(DynamicMsg { msg, dependencies })
    }

    pub fn msg(&self) -> &Msg {
        &self.msg
    }

    pub fn dependency(&self, path: &MessagePath) -> Option<&Msg> {
        self.dependencies.get(path)
    }

    pub fn from_headers(headers: HashMap<String, String>) -> Result<Self> {
        let message_type = headers.get("type").chain_err(|| "Missing header `type`")?;
        let message_definition = headers
            .get("message_definition")
            .chain_err(|| "Missing header `message_definition`")?;
        Self::new(message_type, message_definition)
    }

    fn parse_msg(message_type: &str, message_src: &str) -> Result<Msg> {
        let message_path = message_type
            .try_into()
            .chain_err(|| format!("Message type {} is invalid", message_type))?;
        Msg::new(message_path, message_src)
            .chain_err(|| format!("Failed to parse message {}", message_type))
    }

    fn parse_dependency(message_body: &str) -> Result<Msg> {
        lazy_static! {
            static ref RE_DESCRIPTOR_MSG_TYPE: regex::Regex =
                regex::Regex::new(r#"^\s*MSG:\s*(\S+)\s*$"#).unwrap();
        }
        let message_body = message_body.trim();
        let (message_type_line, message_src) = message_body
            .split_once('\n')
            .chain_err(|| "Message dependency is missing type declaration")?;
        let cap = RE_DESCRIPTOR_MSG_TYPE
            .captures(message_type_line)
            .chain_err(|| format!("Failed to parse message type line `{}`", message_type_line))?;
        let message_type = cap
            .get(1)
            .chain_err(|| format!("Failed to parse message type line `{}`", message_type_line))?;
        Self::parse_msg(message_type.as_str(), message_src)
    }

    pub fn encode(&self, value: &MessageValue, mut w: impl io::Write) -> io::Result<()> {
        self.encode_message(&self.msg, value, &mut w)
    }

    pub fn decode(&self, mut r: impl io::Read) -> io::Result<MessageValue> {
        self.decode_message(&self.msg, &mut r)
    }

    fn get_dependency(&self, path: &MessagePath) -> io::Result<&Msg> {
        self.dependencies.get(path).ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::Other,
                format!("Missing message dependency: {}", path),
            )
        })
    }

    fn encode_message(
        &self,
        msg: &Msg,
        value: &MessageValue,
        w: &mut impl io::Write,
    ) -> io::Result<()> {
        for field in msg.fields() {
            match field.case() {
                FieldCase::Const(_) => continue,
                FieldCase::Unit => {
                    let field_value = get_field(value, field.name())?;
                    self.encode_field(msg.path(), field, field_value, w)?;
                }
                FieldCase::Vector => {
                    let field_value = get_field(value, field.name())?;
                    self.encode_field_array(msg.path(), field, field_value, None, w)?;
                }
                FieldCase::Array(l) => {
                    let field_value = get_field(value, field.name())?;
                    self.encode_field_array(msg.path(), field, field_value, Some(*l), w)?;
                }
            }
        }
        Ok(())
    }

    fn encode_field(
        &self,
        parent: &MessagePath,
        field: &FieldInfo,
        value: &Value,
        w: &mut impl std::io::Write,
    ) -> io::Result<()> {
        match (field.datatype(), value) {
            (DataType::Bool, Value::Bool(v)) => v.encode(w),
            (DataType::I8(_), Value::I8(v)) => v.encode(w),
            (DataType::I16, Value::I16(v)) => v.encode(w),
            (DataType::I32, Value::I32(v)) => v.encode(w),
            (DataType::I64, Value::I64(v)) => v.encode(w),
            (DataType::U8(_), Value::U8(v)) => v.encode(w),
            (DataType::U16, Value::U16(v)) => v.encode(w),
            (DataType::U32, Value::U32(v)) => v.encode(w),
            (DataType::U64, Value::U64(v)) => v.encode(w),
            (DataType::F32, Value::F32(v)) => v.encode(w),
            (DataType::F64, Value::F64(v)) => v.encode(w),
            (DataType::String, Value::String(v)) => v.encode(w),
            (DataType::Time, Value::Time(time)) => time.encode(w),
            (DataType::Duration, Value::Duration(duration)) => duration.encode(w),
            (DataType::LocalMessage(name), Value::Message(v)) => {
                let path = parent.peer(name);
                let dependency = self.get_dependency(&path)?;
                self.encode_message(dependency, v, w)
            }
            (DataType::GlobalMessage(path), Value::Message(v)) => {
                let dependency = self.get_dependency(path)?;
                self.encode_message(dependency, v, w)
            }
            _ => Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Passed in dynamic data value does not match message format",
            )),
        }
    }

    fn encode_field_array(
        &self,
        parent: &MessagePath,
        field: &FieldInfo,
        value: &Value,
        array_length: Option<usize>,
        w: &mut impl std::io::Write,
    ) -> io::Result<()> {
        let value = match value {
            Value::Array(v) => v,
            Value::Bool(_)
            | Value::I8(_)
            | Value::I16(_)
            | Value::I32(_)
            | Value::I64(_)
            | Value::U8(_)
            | Value::U16(_)
            | Value::U32(_)
            | Value::U64(_)
            | Value::F32(_)
            | Value::F64(_)
            | Value::String(_)
            | Value::Time { .. }
            | Value::Duration { .. }
            | Value::Message(_) => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "Passed in dynamic message field is not an array",
                ));
            }
        };
        match array_length {
            Some(array_length) => {
                if array_length != value.len() {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        "Passed in dynamic message array field has wrong length",
                    ));
                }
            }
            None => {
                (value.len() as u32).encode(w.by_ref())?;
            }
        }
        for value in value {
            self.encode_field(parent, field, value, w.by_ref())?;
        }
        Ok(())
    }

    fn decode_message(&self, msg: &Msg, r: &mut impl io::Read) -> io::Result<MessageValue> {
        let mut output = MessageValue::new();
        for field in msg.fields() {
            let value = match field.case() {
                FieldCase::Const(_) => continue,
                FieldCase::Unit => self.decode_field(msg.path(), field, r)?,
                FieldCase::Vector => self.decode_field_array(msg.path(), field, None, r)?,
                FieldCase::Array(l) => self.decode_field_array(msg.path(), field, Some(*l), r)?,
            };
            output.insert(field.name().into(), value);
        }
        Ok(output)
    }

    fn decode_field(
        &self,
        parent: &MessagePath,
        field: &FieldInfo,
        r: &mut impl io::Read,
    ) -> io::Result<Value> {
        Ok(match field.datatype() {
            DataType::Bool => bool::decode(r)?.into(),
            DataType::I8(_) => i8::decode(r)?.into(),
            DataType::I16 => i16::decode(r)?.into(),
            DataType::I32 => i32::decode(r)?.into(),
            DataType::I64 => i64::decode(r)?.into(),
            DataType::U8(_) => u8::decode(r)?.into(),
            DataType::U16 => u16::decode(r)?.into(),
            DataType::U32 => u32::decode(r)?.into(),
            DataType::U64 => u64::decode(r)?.into(),
            DataType::F32 => f32::decode(r)?.into(),
            DataType::F64 => f64::decode(r)?.into(),
            DataType::String => String::decode(r)?.into(),
            DataType::Time => Time::decode(r)?.into(),
            DataType::Duration => Duration::decode(r)?.into(),
            DataType::LocalMessage(name) => {
                let path = parent.peer(name);
                let dependency = self.get_dependency(&path)?;
                self.decode_message(dependency, r)?.into()
            }
            DataType::GlobalMessage(path) => {
                let dependency = self.get_dependency(path)?;
                self.decode_message(dependency, r)?.into()
            }
        })
    }

    fn decode_field_array(
        &self,
        parent: &MessagePath,
        field: &FieldInfo,
        array_length: Option<usize>,
        r: &mut impl io::Read,
    ) -> io::Result<Value> {
        let array_length = match array_length {
            Some(v) => v,
            None => u32::decode(r.by_ref())? as usize,
        };
        // TODO: optimize by checking data type only once
        (0..array_length)
            .map(|_| self.decode_field(parent, field, r))
            .collect()
    }
}
