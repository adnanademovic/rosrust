use crate::{
    parse_msg::match_lines, DataType, Error, FieldCase, FieldInfo, MessagePath, Result, Value,
};
use serde_derive::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use std::fmt::Formatter;

/// A ROS message parsed from a `msg` file.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Msg {
    path: MessagePath,
    fields: Vec<FieldInfo>,
    source: String,
}

impl fmt::Display for Msg {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.source.fmt(f)
    }
}

fn parse_constant<T: std::str::FromStr>(name: &str, value: &str) -> Result<T> {
    value.parse().map_err(|_| Error::BadConstant {
        name: name.into(),
        value: value.into(),
    })
}

impl Msg {
    /// Create a message from a passed in path and source.
    ///
    /// # Errors
    ///
    /// Returns an error if there is an error parsing the message source.
    ///
    /// # Examples
    ///
    /// ```
    /// # use ros_message::Msg;
    /// # use std::convert::TryInto;
    /// #
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let message = Msg::new(
    ///     "foo/Bar".try_into()?,
    ///     r#"# a comment that is ignored
    ///     Header header
    ///     uint32 a
    ///     byte[16] b
    ///     geometry_msgs/Point[] point
    ///     uint32 FOO=5
    ///     string SOME_TEXT=this is # some text, don't be fooled by the hash
    ///     "#,
    /// )?;
    ///
    /// assert_eq!(message.path(), &"foo/Bar".try_into()?);
    /// assert_eq!(message.fields().len(), 6);
    /// # Ok(())
    /// # }
    /// ```
    pub fn new(path: MessagePath, source: &str) -> Result<Msg> {
        let source = source.trim().to_owned();
        let fields = match_lines(&source)?;
        Ok(Msg {
            path,
            fields,
            source,
        })
    }

    /// Returns a map of all constant fields inside the message, with their values parsed.
    ///
    /// # Errors
    ///
    /// Returns an error if there was an issue parsing any of the constants.
    ///
    /// # Examples
    ///
    /// ```
    /// # use ros_message::{Msg, Value};
    /// # use std::convert::TryInto;
    /// #
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let message = Msg::new(
    ///     "foo/Bar".try_into()?,
    ///     r#"# a comment that is ignored
    ///     Header header
    ///     uint32 a
    ///     byte[16] b
    ///     geometry_msgs/Point[] point
    ///     uint32 FOO=5
    ///     string SOME_TEXT=this is # some text, don't be fooled by the hash
    ///     "#,
    /// )?;
    ///
    /// let constants = message.constants()?;
    ///
    /// assert_eq!(constants.len(), 2);
    /// assert_eq!(constants.get("FOO"), Some(&Value::U32(5)));
    /// assert_eq!(
    ///     constants.get("SOME_TEXT"),
    ///     Some(&Value::String("this is # some text, don't be fooled by the hash".into())),
    /// );
    /// # Ok(())
    /// # }
    /// ```
    pub fn constants(&self) -> Result<HashMap<String, Value>> {
        let mut values = HashMap::new();
        for field in &self.fields {
            let raw_value = match field.case() {
                FieldCase::Const(v) => v,
                FieldCase::Unit | FieldCase::Vector | FieldCase::Array(_) => continue,
            };
            let value = match field.datatype() {
                DataType::Bool => Value::Bool(raw_value != "0"),
                DataType::I8(_) => Value::I8(parse_constant(field.name(), raw_value)?),
                DataType::I16 => Value::I16(parse_constant(field.name(), raw_value)?),
                DataType::I32 => Value::I32(parse_constant(field.name(), raw_value)?),
                DataType::I64 => Value::I64(parse_constant(field.name(), raw_value)?),
                DataType::U8(_) => Value::U8(parse_constant(field.name(), raw_value)?),
                DataType::U16 => Value::U16(parse_constant(field.name(), raw_value)?),
                DataType::U32 => Value::U32(parse_constant(field.name(), raw_value)?),
                DataType::U64 => Value::U64(parse_constant(field.name(), raw_value)?),
                DataType::F32 => Value::F32(parse_constant(field.name(), raw_value)?),
                DataType::F64 => Value::F64(parse_constant(field.name(), raw_value)?),
                DataType::String => Value::String(raw_value.clone()),
                DataType::Time
                | DataType::Duration
                | DataType::LocalMessage(_)
                | DataType::GlobalMessage(_) => continue,
            };
            values.insert(field.name().into(), value);
        }
        Ok(values)
    }

    /// Returns the path of the message.
    pub fn path(&self) -> &MessagePath {
        &self.path
    }

    /// Returns a slice of all fields.
    pub fn fields(&self) -> &[FieldInfo] {
        &self.fields
    }

    /// Returns the original source.
    pub fn source(&self) -> &str {
        &self.source
    }

    /// Returns a all message paths that this message directly depends upon.
    ///
    /// They are listed in the order that they appear in in the message, and duplicates
    /// are allowed.
    ///
    /// Indirect dependencies are not included, and if you want an exhaustive list of all
    /// dependencies, you have to manually traverse every message being depended upon.
    /// # Examples
    ///
    /// ```
    /// # use ros_message::Msg;
    /// # use std::convert::TryInto;
    /// #
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let message = Msg::new(
    ///     "foo/Bar".try_into()?,
    ///     r#"
    ///     Header header
    ///     geometry_msgs/Point[] point1
    ///     Point[] point2
    ///     foo/Point[] point2_but_with_global_path
    ///     foo/Baz[] baz
    ///     "#,
    /// )?;
    ///
    /// let dependencies = message.dependencies();
    ///
    /// assert_eq!(dependencies, vec![
    ///     "std_msgs/Header".try_into()?,
    ///     "geometry_msgs/Point".try_into()?,
    ///     "foo/Point".try_into()?,
    ///     "foo/Point".try_into()?,
    ///     "foo/Baz".try_into()?,
    /// ]);
    /// # Ok(())
    /// # }
    /// ```
    pub fn dependencies(&self) -> Vec<MessagePath> {
        self.fields
            .iter()
            .filter_map(|field| match field.datatype() {
                DataType::LocalMessage(ref name) => Some(self.path.peer(name)),
                DataType::GlobalMessage(ref message) => Some(message.clone()),
                _ => None,
            })
            .collect()
    }

    /// Returns the MD5 sum of this message.
    ///
    /// Any direct dependency must have its MD5 sum provided in the passed in hashes.
    ///
    /// All direct dependencies are returned by the `dependencies()` method.
    ///
    /// # Errors
    ///
    /// An error is returned if some dependency is missing in the hashes.
    #[cfg(test)]
    pub fn calculate_md5(&self, hashes: &HashMap<MessagePath, String>) -> Result<String> {
        use md5::{Digest, Md5};

        let mut hasher = Md5::new();
        hasher.update(&self.get_md5_representation(hashes)?);
        Ok(hex::encode(hasher.finalize()))
    }

    /// Returns the full MD5 representation of the message.
    ///
    /// This is the string that is sent to the MD5 hasher to digest.
    ///
    /// # Errors
    ///
    /// An error is returned if some dependency is missing in the hashes.
    ///
    /// # Examples
    ///
    /// ```
    /// # use ros_message::Msg;
    /// # use std::convert::TryInto;
    /// # use std::collections::HashMap;
    /// #
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let message = Msg::new(
    ///     "foo/Bar".try_into()?,
    ///     r#"# a comment that is ignored
    ///     Header header
    ///     uint32 a
    ///     byte[16] b
    ///     geometry_msgs/Point[] point
    ///     Baz baz
    ///     uint32 FOO=5
    ///     string SOME_TEXT=this is # some text, don't be fooled by the hash
    ///     "#,
    /// )?;
    ///
    /// let mut hashes = HashMap::new();
    /// hashes.insert("std_msgs/Header".try_into()?, "hash1".into());
    /// hashes.insert("geometry_msgs/Point".try_into()?, "hash2".into());
    /// hashes.insert("foo/Baz".try_into()?, "hash3".into());
    ///
    /// let representation = message.get_md5_representation(&hashes)?;
    ///
    /// assert_eq!(
    ///     representation,
    /// r#"uint32 FOO=5
    /// string SOME_TEXT=this is # some text, don't be fooled by the hash
    /// hash1 header
    /// uint32 a
    /// byte[16] b
    /// hash2 point
    /// hash3 baz"#);
    /// # Ok(())
    /// # }
    /// ```
    pub fn get_md5_representation(&self, hashes: &HashMap<MessagePath, String>) -> Result<String> {
        let constants = self
            .fields
            .iter()
            .filter(|v| v.is_constant())
            .map(|v| v.md5_string(self.path.package(), hashes))
            .collect::<Result<Vec<String>>>()?;
        let fields = self
            .fields
            .iter()
            .filter(|v| !v.is_constant())
            .map(|v| v.md5_string(self.path.package(), hashes))
            .collect::<Result<Vec<String>>>()?;
        let representation = constants
            .into_iter()
            .chain(fields)
            .collect::<Vec<_>>()
            .join("\n");
        Ok(representation)
    }

    /// Returns true if the message has a header field.
    ///
    /// A header field is a unit value named `header` of type `std_msgs/Header`.
    /// The package can be elided in this special case, no matter the package that
    /// the containing message is located in.
    pub fn has_header(&self) -> bool {
        self.fields.iter().any(FieldInfo::is_header)
    }
}
