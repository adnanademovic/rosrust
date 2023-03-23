use crate::{DataType, Error, MessagePath, Result, Value};
use serde_derive::{Deserialize, Serialize};
use std::collections::HashMap;
use std::convert::{TryFrom, TryInto};
use std::fmt;
use std::fmt::Formatter;
use std::hash::{Hash, Hasher};

/// Represents all possible variants of a message field
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FieldCase {
    /// Field of a single item.
    ///
    /// Examples: `float32`, `geometry_msgs/Point`.
    Unit,
    /// Field of an arbitrary length array.
    ///
    /// Examples: `float32[]`, `geometry_msgs/Point[]`.
    Vector,
    /// Field of a fixed length array.
    ///
    /// The contained number is the array length.
    ///
    /// Examples: `float32[64]`, `geometry_msgs/Point[10]`.
    Array(usize),
    /// Field describing a constant value.
    ///
    /// The contained `String` is the unparsed value.
    ///
    /// Example: `float32 FOO=123.4`.
    Const(String),
}

#[derive(Clone, Debug)]
struct Uncompared<T> {
    inner: T,
}

impl<T> Hash for Uncompared<T> {
    fn hash<H: Hasher>(&self, _state: &mut H) {}
}

impl<T> PartialEq for Uncompared<T> {
    fn eq(&self, _other: &Self) -> bool {
        true
    }
}

impl<T> Eq for Uncompared<T> {}

/// Full description of one field in a `msg` or `srv` file.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(into = "FieldInfoSerde")]
#[serde(try_from = "FieldInfoSerde")]
pub struct FieldInfo {
    datatype: DataType,
    name: String,
    case: FieldCase,
    const_value: Uncompared<Option<Value>>,
}

impl fmt::Display for FieldInfo {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match &self.case {
            FieldCase::Unit => write!(f, "{} {}", self.datatype, self.name),
            FieldCase::Vector => write!(f, "{}[] {}", self.datatype, self.name),
            FieldCase::Array(l) => write!(f, "{}[{}] {}", self.datatype, l, self.name),
            FieldCase::Const(val) => write!(f, "{} {}={}", self.datatype, self.name, val),
        }
    }
}

impl FieldInfo {
    /// Create a field of the provided type, name and variant.
    ///
    /// # Errors
    ///
    /// An error will be returned if the data type cannot be parsed, or const data is invalid.
    ///
    /// # Examples
    ///
    /// ```
    /// # use ros_message::{DataType, FieldInfo, FieldCase};
    /// #
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let field = FieldInfo::new("int16", "foo", FieldCase::Vector)?;
    ///
    /// assert_eq!(field.name(), "foo");
    /// assert_eq!(field.datatype(), &DataType::I16);
    /// assert_eq!(field.case(), &FieldCase::Vector);
    /// assert_eq!(format!("{}", field), "int16[] foo");
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// ```
    /// # use ros_message::{FieldInfo, FieldCase};
    /// assert!(FieldInfo::new("bad/field/type", "foo", FieldCase::Vector).is_err());
    /// ```
    pub fn new(datatype: &str, name: impl Into<String>, case: FieldCase) -> Result<FieldInfo> {
        Self::evaluate(datatype.try_into()?, name.into(), case)
    }

    fn evaluate(datatype: DataType, name: String, case: FieldCase) -> Result<FieldInfo> {
        let const_value = match &case {
            FieldCase::Const(raw_value) => Some(
                match &datatype {
                    DataType::Bool => Some(Value::Bool(raw_value != "0")),
                    DataType::I8(_) => raw_value.parse().ok().map(Value::I8),
                    DataType::I16 => raw_value.parse().ok().map(Value::I16),
                    DataType::I32 => raw_value.parse().ok().map(Value::I32),
                    DataType::I64 => raw_value.parse().ok().map(Value::I64),
                    DataType::U8(_) => raw_value.parse().ok().map(Value::U8),
                    DataType::U16 => raw_value.parse().ok().map(Value::U16),
                    DataType::U32 => raw_value.parse().ok().map(Value::U32),
                    DataType::U64 => raw_value.parse().ok().map(Value::U64),
                    DataType::F32 => raw_value.parse().ok().map(Value::F32),
                    DataType::F64 => raw_value.parse().ok().map(Value::F64),
                    DataType::String => Some(Value::String(raw_value.clone())),
                    DataType::Time
                    | DataType::Duration
                    | DataType::LocalMessage(_)
                    | DataType::GlobalMessage(_) => None,
                }
                .ok_or_else(|| Error::BadConstant {
                    name: name.clone(),
                    datatype: format!("{}", datatype),
                    value: raw_value.into(),
                })?,
            ),
            FieldCase::Unit | FieldCase::Vector | FieldCase::Array(_) => None,
        };
        Ok(FieldInfo {
            datatype,
            name,
            case,
            const_value: Uncompared { inner: const_value },
        })
    }

    /// Returns the data type of the field.
    pub fn datatype(&self) -> &DataType {
        &self.datatype
    }

    /// Returns the name of the field.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the case of the field.
    pub fn case(&self) -> &FieldCase {
        &self.case
    }

    /// Returns the stored value if a constant field.
    pub fn const_value(&self) -> Option<&Value> {
        self.const_value.inner.as_ref()
    }

    /// Returns true if the field contains a constant value.
    ///
    /// # Examples
    ///
    /// ```
    /// # use ros_message::{DataType, FieldInfo, FieldCase};
    /// #
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// assert!(!FieldInfo::new("int16", "foo", FieldCase::Vector)?.is_constant());
    /// assert!(FieldInfo::new("int16", "foo", FieldCase::Const("12".into()))?.is_constant());
    /// # Ok(())
    /// # }
    /// ```
    pub fn is_constant(&self) -> bool {
        matches!(self.case, FieldCase::Const(..))
    }

    /// Returns the representation of the data type when constructing the MD5 sum.
    ///
    /// For built in types, it is the same as the message row, but with consistent whitespace.
    ///
    /// For message types, the type is replaced with the message's MD5 sum,
    /// which is passed in via the `hashes` argument.
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
    /// # use ros_message::{FieldInfo, FieldCase};
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
    /// assert_eq!(
    ///     FieldInfo::new("int16", "foo", FieldCase::Unit)?.md5_string("foo", &hashes)?,
    ///     "int16 foo",
    /// );
    /// assert_eq!(
    ///     FieldInfo::new("float64", "foo", FieldCase::Vector)?.md5_string("foo", &hashes)?,
    ///     "float64[] foo",
    /// );
    /// assert_eq!(
    ///     FieldInfo::new("byte", "foo", FieldCase::Array(12))?.md5_string("foo", &hashes)?,
    ///     "byte[12] foo",
    /// );
    /// assert_eq!(
    ///     FieldInfo::new("byte", "FOO", FieldCase::Const("12".into()))?.md5_string("foo", &hashes)?,
    ///     "byte FOO=12",
    /// );
    /// assert_eq!(
    ///     FieldInfo::new("Header", "foo", FieldCase::Unit)?.md5_string("foo", &hashes)?,
    ///     "123 foo",
    /// );
    /// assert_eq!(
    ///     FieldInfo::new("Position", "foo", FieldCase::Vector)?.md5_string("foo", &hashes)?,
    ///     "678 foo",
    /// );
    /// assert_eq!(
    ///     FieldInfo::new("geometry_msgs/Position", "foo", FieldCase::Array(12))?.md5_string("foo", &hashes)?,
    ///     "345 foo",
    /// );
    /// assert!(
    ///     FieldInfo::new("other_msgs/Position", "foo", FieldCase::Unit)?
    ///         .md5_string("foo", &hashes)
    ///         .is_err(),
    /// );
    /// # Ok(())
    /// # }
    /// ```
    pub fn md5_string(
        &self,
        package: &str,
        hashes: &HashMap<MessagePath, String>,
    ) -> Result<String> {
        let datatype = self.datatype.md5_str(package, hashes)?;
        Ok(match (self.datatype.is_builtin(), &self.case) {
            (_, FieldCase::Const(v)) => format!("{} {}={}", datatype, self.name, v),
            (false, _) | (_, &FieldCase::Unit) => format!("{} {}", datatype, self.name),
            (true, &FieldCase::Vector) => format!("{}[] {}", datatype, self.name),
            (true, &FieldCase::Array(l)) => format!("{}[{}] {}", datatype, l, self.name),
        })
    }

    /// Returns true if this is a header field.
    ///
    /// The header field is special, being a unit value of type `std_msgs/Header`
    /// and named `header`. Also in this special case, the package can be elided,
    /// even if we're not in the same package.
    ///
    /// If any of those requirements are not met, it is not a header field.
    ///
    /// The field is special because ROS channel publishers are allowed to populate it with
    /// the node and publisher specific data.
    ///
    /// # Examples
    ///
    /// ```
    /// # use ros_message::{FieldInfo, FieldCase};
    /// #
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// assert!(FieldInfo::new("Header", "header", FieldCase::Unit)?.is_header());
    /// assert!(FieldInfo::new("std_msgs/Header", "header", FieldCase::Unit)?.is_header());
    /// assert!(!FieldInfo::new("Header", "header", FieldCase::Vector)?.is_header());
    /// assert!(!FieldInfo::new("Header", "header", FieldCase::Array(5))?.is_header());
    /// assert!(FieldInfo::new("Header", "header", FieldCase::Const("12".into())).is_err());
    /// assert!(!FieldInfo::new("Header", "some_field", FieldCase::Unit)?.is_header());
    /// # Ok(())
    /// # }
    /// ```
    pub fn is_header(&self) -> bool {
        if self.case != FieldCase::Unit || self.name != "header" {
            return false;
        }
        match &self.datatype {
            DataType::GlobalMessage(msg) => msg.package() == "std_msgs" && msg.name() == "Header",
            _ => false,
        }
    }
}

#[derive(Serialize, Deserialize)]
struct FieldInfoSerde {
    datatype: DataType,
    name: String,
    case: FieldCase,
}

impl TryFrom<FieldInfoSerde> for FieldInfo {
    type Error = Error;

    fn try_from(src: FieldInfoSerde) -> Result<Self> {
        Self::evaluate(src.datatype, src.name, src.case)
    }
}

impl From<FieldInfo> for FieldInfoSerde {
    fn from(src: FieldInfo) -> Self {
        Self {
            datatype: src.datatype,
            name: src.name,
            case: src.case,
        }
    }
}
