use crate::{DataType, MessagePath, Result};
use serde_derive::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use std::fmt::Formatter;

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FieldCase {
    Unit,
    Vector,
    Array(usize),
    Const(String),
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct FieldInfo {
    datatype: DataType,
    name: String,
    case: FieldCase,
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
    pub fn new(datatype: &str, name: impl Into<String>, case: FieldCase) -> Result<FieldInfo> {
        Ok(FieldInfo {
            datatype: DataType::parse(datatype)?,
            name: name.into(),
            case,
        })
    }

    pub fn datatype(&self) -> &DataType {
        &self.datatype
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn case(&self) -> &FieldCase {
        &self.case
    }

    pub fn is_constant(&self) -> bool {
        matches!(self.case, FieldCase::Const(..))
    }

    pub fn md5_string(
        &self,
        package: &str,
        hashes: &HashMap<MessagePath, String>,
    ) -> Result<String> {
        let datatype = self.datatype.md5_string(package, hashes)?;
        Ok(match (self.datatype.is_builtin(), &self.case) {
            (_, &FieldCase::Const(ref v)) => format!("{} {}={}", datatype, self.name, v),
            (false, _) | (_, &FieldCase::Unit) => format!("{} {}", datatype, self.name),
            (true, &FieldCase::Vector) => format!("{}[] {}", datatype, self.name),
            (true, &FieldCase::Array(l)) => format!("{}[{}] {}", datatype, l, self.name),
        })
    }

    pub fn is_header(&self) -> bool {
        if self.case != FieldCase::Unit || self.name != "header" {
            return false;
        }
        match &self.datatype {
            DataType::RemoteStruct(msg) => msg.package() == "std_msgs" && msg.name() == "Header",
            _ => false,
        }
    }
}
