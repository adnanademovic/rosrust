use crate::{Duration, Time};
use itertools::Itertools;
use serde_derive::{Deserialize, Serialize};
use std::collections::HashMap;
use std::convert::{TryFrom, TryInto};
use std::fmt;
use std::fmt::{Display, Formatter};
use std::iter::FromIterator;

/// Represents an arbitrary ROS message.
pub type MessageValue = HashMap<String, Value>;

/// Represents an arbitrary ROS message or value in it.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Value {
    /// Represents `bool`.
    Bool(bool),
    /// Represents `int8` or `byte`.
    I8(i8),
    /// Represents `int16`.
    I16(i16),
    /// Represents `int32`.
    I32(i32),
    /// Represents `int64`.
    I64(i64),
    /// Represents `uint8` or `char`.
    U8(u8),
    /// Represents `uint16`.
    U16(u16),
    /// Represents `uint32`.
    U32(u32),
    /// Represents `uint64`.
    U64(u64),
    /// Represents `float32`.
    F32(f32),
    /// Represents `float64`.
    F64(f64),
    /// Represents `string`.
    String(String),
    /// Represents `time`.
    Time(Time),
    /// Represents `duration`.
    Duration(Duration),
    /// Represents `some_type[]` or `some_type[length]`.
    ///
    /// For example: `float32[64]`, `geometry_msgs/Point[]`.
    Array(Vec<Value>),
    /// Represents an embedded message.
    Message(MessageValue),
}

impl Value {
    fn fmt_indented(&self, indentation: usize, step: usize, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Value::Bool(v) => v.fmt(f),
            Value::I8(v) => v.fmt(f),
            Value::I16(v) => v.fmt(f),
            Value::I32(v) => v.fmt(f),
            Value::I64(v) => v.fmt(f),
            Value::U8(v) => v.fmt(f),
            Value::U16(v) => v.fmt(f),
            Value::U32(v) => v.fmt(f),
            Value::U64(v) => v.fmt(f),
            Value::F32(v) => v.fmt(f),
            Value::F64(v) => v.fmt(f),
            Value::String(v) => write!(f, "{:?}", v),
            Value::Time(v) => v.fmt(f),
            Value::Duration(v) => v.fmt(f),
            Value::Array(items) => {
                for item in items {
                    writeln!(f)?;
                    write!(f, "{:indent$}- ", "", indent = indentation)?;
                    item.fmt_indented(indentation + step, step, f)?;
                }
                Ok(())
            }
            Value::Message(items) => {
                for (key, item) in items.iter().sorted_by(|a, b| Ord::cmp(&a.0, &b.0)) {
                    writeln!(f)?;
                    write!(f, "{:indent$}{}: ", "", key, indent = indentation)?;
                    item.fmt_indented(indentation + step, step, f)?;
                }
                Ok(())
            }
        }
    }

    /// Returns the content if `Value` is a `bool`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use ros_message::Value;
    /// assert_eq!(Value::Bool(true).as_bool(), Some(true));
    /// assert_eq!(Value::Bool(false).as_bool(), Some(false));
    /// assert!(Value::U32(12).as_bool().is_none());
    /// ```
    pub fn as_bool(&self) -> Option<bool> {
        if let Value::Bool(value) = self {
            Some(*value)
        } else {
            None
        }
    }

    /// Returns the content if `Value` is an `i8`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use ros_message::Value;
    /// assert_eq!(Value::I8(12).as_i8(), Some(12));
    /// assert!(Value::U32(12).as_i8().is_none());
    /// ```
    pub fn as_i8(&self) -> Option<i8> {
        if let Value::I8(value) = self {
            Some(*value)
        } else {
            None
        }
    }

    /// Returns the content if `Value` is an `i16`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use ros_message::Value;
    /// assert_eq!(Value::I16(12).as_i16(), Some(12));
    /// assert!(Value::U32(12).as_i16().is_none());
    /// ```
    pub fn as_i16(&self) -> Option<i16> {
        if let Value::I16(value) = self {
            Some(*value)
        } else {
            None
        }
    }

    /// Returns the content if `Value` is an `i32`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use ros_message::Value;
    /// assert_eq!(Value::I32(12).as_i32(), Some(12));
    /// assert!(Value::U32(12).as_i32().is_none());
    /// ```
    pub fn as_i32(&self) -> Option<i32> {
        if let Value::I32(value) = self {
            Some(*value)
        } else {
            None
        }
    }

    /// Returns the content if `Value` is an `i64`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use ros_message::Value;
    /// assert_eq!(Value::I64(12).as_i64(), Some(12));
    /// assert!(Value::U32(12).as_i64().is_none());
    /// ```
    pub fn as_i64(&self) -> Option<i64> {
        if let Value::I64(value) = self {
            Some(*value)
        } else {
            None
        }
    }

    /// Returns the content if `Value` is a `u8`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use ros_message::Value;
    /// assert_eq!(Value::U8(12).as_u8(), Some(12));
    /// assert!(Value::U32(12).as_u8().is_none());
    /// ```
    pub fn as_u8(&self) -> Option<u8> {
        if let Value::U8(value) = self {
            Some(*value)
        } else {
            None
        }
    }

    /// Returns the content if `Value` is a `u16`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use ros_message::Value;
    /// assert_eq!(Value::U16(12).as_u16(), Some(12));
    /// assert!(Value::U32(12).as_u16().is_none());
    /// ```
    pub fn as_u16(&self) -> Option<u16> {
        if let Value::U16(value) = self {
            Some(*value)
        } else {
            None
        }
    }

    /// Returns the content if `Value` is a `u32`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use ros_message::Value;
    /// assert_eq!(Value::U32(12).as_u32(), Some(12));
    /// assert!(Value::U16(12).as_u32().is_none());
    /// ```
    pub fn as_u32(&self) -> Option<u32> {
        if let Value::U32(value) = self {
            Some(*value)
        } else {
            None
        }
    }

    /// Returns the content if `Value` is a `u64`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use ros_message::Value;
    /// assert_eq!(Value::U64(12).as_u64(), Some(12));
    /// assert!(Value::U32(12).as_u64().is_none());
    /// ```
    pub fn as_u64(&self) -> Option<u64> {
        if let Value::U64(value) = self {
            Some(*value)
        } else {
            None
        }
    }

    /// Returns the content if `Value` is an `f32`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use ros_message::Value;
    /// assert_eq!(Value::F32(12.0).as_f32(), Some(12.0));
    /// assert!(Value::U32(12).as_f32().is_none());
    /// ```
    pub fn as_f32(&self) -> Option<f32> {
        if let Value::F32(value) = self {
            Some(*value)
        } else {
            None
        }
    }

    /// Returns the content if `Value` is an `f64`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use ros_message::Value;
    /// assert_eq!(Value::F64(12.0).as_f64(), Some(12.0));
    /// assert!(Value::U32(12).as_f64().is_none());
    /// ```
    pub fn as_f64(&self) -> Option<f64> {
        if let Value::F64(value) = self {
            Some(*value)
        } else {
            None
        }
    }

    /// Returns a `&str` if `Value` is a `String`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use ros_message::Value;
    /// assert_eq!(Value::String("foo".into()).as_str(), Some("foo"));
    /// assert!(Value::U32(12).as_str().is_none());
    /// ```
    pub fn as_str(&self) -> Option<&str> {
        if let Value::String(value) = self {
            Some(value)
        } else {
            None
        }
    }

    /// Returns the content if `Value` is a `String`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use ros_message::Value;
    /// assert_eq!(Value::String("foo".into()).try_into_string(), Some("foo".into()));
    /// assert!(Value::U32(12).try_into_string().is_none());
    /// ```
    pub fn try_into_string(self) -> Option<String> {
        if let Value::String(value) = self {
            Some(value)
        } else {
            None
        }
    }

    /// Returns the content if `Value` is a `Time` struct.
    ///
    /// # Examples
    ///
    /// ```
    /// # use ros_message::{Time, Value};
    /// assert_eq!(
    ///     Value::Time(Time::from_nanos(120)).as_time(),
    ///     Some(Time::from_nanos(120)),
    /// );
    /// assert!(Value::U32(12).as_time().is_none());
    /// ```
    pub fn as_time(&self) -> Option<Time> {
        if let Value::Time(value) = self {
            Some(*value)
        } else {
            None
        }
    }

    /// Returns the content if `Value` is a `Duration` struct.
    ///
    /// # Examples
    ///
    /// ```
    /// # use ros_message::{Duration, Value};
    /// assert_eq!(
    ///     Value::Duration(Duration::from_nanos(120)).as_duration(),
    ///     Some(Duration::from_nanos(120)),
    /// );
    /// assert!(Value::U32(12).as_duration().is_none());
    /// ```
    pub fn as_duration(&self) -> Option<Duration> {
        if let Value::Duration(value) = self {
            Some(*value)
        } else {
            None
        }
    }

    /// Returns a reference to the content if `Value` is an array.
    ///
    /// # Examples
    ///
    /// ```
    /// # use ros_message::Value;
    /// assert_eq!(
    ///     Value::Array(vec![1u32.into(), 2u32.into(), 3u32.into()]).as_slice(),
    ///     Some(&[Value::U32(1), Value::U32(2), Value::U32(3)][..]),
    /// );
    /// assert!(Value::U32(12).as_slice().is_none());
    /// ```
    pub fn as_slice(&self) -> Option<&[Value]> {
        if let Value::Array(value) = self {
            Some(value)
        } else {
            None
        }
    }

    /// Returns the content if `Value` is an array.
    ///
    /// # Examples
    ///
    /// ```
    /// # use ros_message::Value;
    /// assert_eq!(
    ///     Value::Array(vec![1u32.into(), 2u32.into(), 3u32.into()]).try_into_vec(),
    ///     Some(vec![Value::U32(1), Value::U32(2), Value::U32(3)]),
    /// );
    /// assert!(Value::U32(12).try_into_vec().is_none());
    /// ```
    pub fn try_into_vec(self) -> Option<Vec<Value>> {
        if let Value::Array(value) = self {
            Some(value)
        } else {
            None
        }
    }

    /// Returns a reference to the content if `Value` is a message.
    ///
    /// # Examples
    ///
    /// ```
    /// # use ros_message::Value;
    /// # use std::collections::HashMap;
    /// let mut data = HashMap::<String, Value>::new();
    /// data.insert("foo".into(), true.into());
    /// data.insert("bar".into(), false.into());
    /// assert_eq!(Value::Message(data.clone()).as_map(), Some(&data));
    /// assert!(Value::U32(12).as_map().is_none());
    /// ```
    pub fn as_map(&self) -> Option<&MessageValue> {
        if let Value::Message(value) = self {
            Some(value)
        } else {
            None
        }
    }

    /// Returns the content if `Value` is a message.
    ///
    /// # Examples
    ///
    /// ```
    /// # use ros_message::Value;
    /// # use std::collections::HashMap;
    /// let mut data = HashMap::<String, Value>::new();
    /// data.insert("foo".into(), true.into());
    /// data.insert("bar".into(), false.into());
    /// assert_eq!(Value::Message(data.clone()).try_into_map(), Some(data));
    /// assert!(Value::U32(12).try_into_map().is_none());
    /// ```
    pub fn try_into_map(self) -> Option<MessageValue> {
        if let Value::Message(value) = self {
            Some(value)
        } else {
            None
        }
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.fmt_indented(0, 2, f)
    }
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

impl<T: Into<Value>> From<Vec<T>> for Value {
    fn from(v: Vec<T>) -> Self {
        Self::Array(v.into_iter().map(Into::into).collect())
    }
}

impl<T: Into<Value>, const L: usize> From<[T; L]> for Value {
    fn from(v: [T; L]) -> Self {
        Self::Array(std::array::IntoIter::new(v).map(Into::into).collect())
    }
}

impl From<HashMap<String, Value>> for Value {
    fn from(v: HashMap<String, Value>) -> Self {
        Self::Message(v)
    }
}

impl TryFrom<Value> for bool {
    type Error = ();

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        value.as_bool().ok_or(())
    }
}

impl TryFrom<Value> for i8 {
    type Error = ();

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        value.as_i8().ok_or(())
    }
}

impl TryFrom<Value> for i16 {
    type Error = ();

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        value.as_i16().ok_or(())
    }
}

impl TryFrom<Value> for i32 {
    type Error = ();

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        value.as_i32().ok_or(())
    }
}

impl TryFrom<Value> for i64 {
    type Error = ();

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        value.as_i64().ok_or(())
    }
}

impl TryFrom<Value> for u8 {
    type Error = ();

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        value.as_u8().ok_or(())
    }
}

impl TryFrom<Value> for u16 {
    type Error = ();

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        value.as_u16().ok_or(())
    }
}

impl TryFrom<Value> for u32 {
    type Error = ();

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        value.as_u32().ok_or(())
    }
}

impl TryFrom<Value> for u64 {
    type Error = ();

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        value.as_u64().ok_or(())
    }
}

impl TryFrom<Value> for f32 {
    type Error = ();

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        value.as_f32().ok_or(())
    }
}

impl TryFrom<Value> for f64 {
    type Error = ();

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        value.as_f64().ok_or(())
    }
}

impl TryFrom<Value> for String {
    type Error = ();

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        value.try_into_string().ok_or(())
    }
}

impl TryFrom<Value> for Time {
    type Error = ();

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        value.as_time().ok_or(())
    }
}

impl TryFrom<Value> for Duration {
    type Error = ();

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        value.as_duration().ok_or(())
    }
}

impl<T: TryFrom<Value>> TryFrom<Value> for Vec<T> {
    type Error = ();

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        let value = value.try_into_vec().ok_or(())?;
        value
            .into_iter()
            .map(TryInto::try_into)
            .collect::<Result<Self, _>>()
            .map_err(|_| ())
    }
}

impl<T: TryFrom<Value>, const L: usize> TryFrom<Value> for [T; L] {
    type Error = ();

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        let value = value.try_into_vec().ok_or(())?;
        if value.len() != L {
            return Err(());
        }
        array_init::from_iter(value.into_iter().filter_map(|v| v.try_into().ok())).ok_or(())
    }
}

impl TryFrom<Value> for HashMap<String, Value> {
    type Error = ();

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        value.try_into_map().ok_or(())
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
