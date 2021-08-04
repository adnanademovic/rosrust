use crate::{DataType, MessagePath, Result};
use std::collections::HashMap;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FieldCase {
    Unit,
    Vector,
    Array(usize),
    Const(String),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FieldInfo {
    pub datatype: DataType,
    pub name: String,
    pub case: FieldCase,
}

impl FieldInfo {
    pub fn new(datatype: &str, name: &str, case: FieldCase) -> Result<FieldInfo> {
        Ok(FieldInfo {
            datatype: DataType::parse(datatype)?,
            name: name.into(),
            case,
        })
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
