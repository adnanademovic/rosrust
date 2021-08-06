use crate::{Duration, Time};
use serde_derive::{Deserialize, Serialize};
use std::collections::HashMap;
use std::iter::FromIterator;

pub type MessageValue = HashMap<String, Value>;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Value {
    Bool(bool),
    I8(i8),
    I16(i16),
    I32(i32),
    I64(i64),
    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
    F32(f32),
    F64(f64),
    String(String),
    Time(Time),
    Duration(Duration),
    Array(Vec<Value>),
    Message(MessageValue),
}

impl From<bool> for Value {
    fn from(v: bool) -> Self {
        Self::Bool(v)
    }
}

impl From<i8> for Value {
    fn from(v: i8) -> Self {
        Self::I8(v)
    }
}

impl From<i16> for Value {
    fn from(v: i16) -> Self {
        Self::I16(v)
    }
}

impl From<i32> for Value {
    fn from(v: i32) -> Self {
        Self::I32(v)
    }
}

impl From<i64> for Value {
    fn from(v: i64) -> Self {
        Self::I64(v)
    }
}

impl From<u8> for Value {
    fn from(v: u8) -> Self {
        Self::U8(v)
    }
}

impl From<u16> for Value {
    fn from(v: u16) -> Self {
        Self::U16(v)
    }
}

impl From<u32> for Value {
    fn from(v: u32) -> Self {
        Self::U32(v)
    }
}

impl From<u64> for Value {
    fn from(v: u64) -> Self {
        Self::U64(v)
    }
}

impl From<f32> for Value {
    fn from(v: f32) -> Self {
        Self::F32(v)
    }
}

impl From<f64> for Value {
    fn from(v: f64) -> Self {
        Self::F64(v)
    }
}

impl From<String> for Value {
    fn from(v: String) -> Self {
        Self::String(v)
    }
}

impl From<Time> for Value {
    fn from(v: Time) -> Self {
        Self::Time(v)
    }
}

impl From<Duration> for Value {
    fn from(v: Duration) -> Self {
        Self::Duration(v)
    }
}

impl From<Vec<Value>> for Value {
    fn from(v: Vec<Value>) -> Self {
        Self::Array(v)
    }
}

impl From<HashMap<String, Value>> for Value {
    fn from(v: HashMap<String, Value>) -> Self {
        Self::Message(v)
    }
}

impl<K: Into<String>, T: Into<Value>> FromIterator<(K, T)> for Value {
    fn from_iter<I: IntoIterator<Item = (K, T)>>(iter: I) -> Self {
        Self::Message(
            iter.into_iter()
                .map(|(key, value)| (key.into(), value.into()))
                .collect(),
        )
    }
}

impl<T: Into<Value>> FromIterator<T> for Value {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        Self::Array(iter.into_iter().map(Into::into).collect())
    }
}
