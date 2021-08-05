use std::collections::HashMap;

use crate::{
    parse_msg::match_lines, DataType, Error, FieldCase, FieldInfo, MessagePath, Result, Value,
};

#[derive(Clone, Debug)]
pub struct Msg {
    pub path: MessagePath,
    pub fields: Vec<FieldInfo>,
    pub source: String,
}

fn parse_constant<T: std::str::FromStr>(name: &str, value: &str) -> Result<T> {
    value.parse().map_err(|_| Error::BadConstant {
        name: name.into(),
        value: value.into(),
    })
}

impl Msg {
    pub fn new(path: MessagePath, source: &str) -> Result<Msg> {
        let fields = match_lines(source)?;
        Ok(Msg {
            path,
            fields,
            source: source.trim().into(),
        })
    }

    pub fn constants(&self) -> Result<HashMap<String, Value>> {
        let mut values = HashMap::new();
        for field in &self.fields {
            let raw_value = match &field.case {
                FieldCase::Const(v) => v,
                FieldCase::Unit | FieldCase::Vector | FieldCase::Array(_) => continue,
            };
            let value = match field.datatype {
                DataType::Bool => Value::Bool(raw_value != "0"),
                DataType::I8(_) => Value::I8(parse_constant(&field.name, raw_value)?),
                DataType::I16 => Value::I16(parse_constant(&field.name, raw_value)?),
                DataType::I32 => Value::I32(parse_constant(&field.name, raw_value)?),
                DataType::I64 => Value::I64(parse_constant(&field.name, raw_value)?),
                DataType::U8(_) => Value::U8(parse_constant(&field.name, raw_value)?),
                DataType::U16 => Value::U16(parse_constant(&field.name, raw_value)?),
                DataType::U32 => Value::U32(parse_constant(&field.name, raw_value)?),
                DataType::U64 => Value::U64(parse_constant(&field.name, raw_value)?),
                DataType::F32 => Value::F32(parse_constant(&field.name, raw_value)?),
                DataType::F64 => Value::F64(parse_constant(&field.name, raw_value)?),
                DataType::String => Value::String(raw_value.clone()),
                DataType::Time
                | DataType::Duration
                | DataType::LocalStruct(_)
                | DataType::RemoteStruct(_) => continue,
            };
            values.insert(field.name.clone(), value);
        }
        Ok(values)
    }

    pub fn full_name(&self) -> String {
        format!("{}/{}", self.path.package(), self.path.name())
    }

    pub fn dependencies(&self) -> Result<Vec<MessagePath>> {
        self.fields
            .iter()
            .filter_map(|field| match field.datatype {
                DataType::LocalStruct(ref name) => {
                    Some(MessagePath::new(self.path.package(), name))
                }
                DataType::RemoteStruct(ref message) => Some(Ok(message.clone())),
                _ => None,
            })
            .collect()
    }

    #[cfg(test)]
    pub fn calculate_md5(&self, hashes: &HashMap<MessagePath, String>) -> Result<String> {
        use md5::{Digest, Md5};

        let mut hasher = Md5::new();
        hasher.update(&self.get_md5_representation(hashes)?);
        Ok(hex::encode(hasher.finalize()))
    }

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

    pub fn has_header(&self) -> bool {
        self.fields.iter().any(FieldInfo::is_header)
    }
}
