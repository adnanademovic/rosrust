use std::collections::HashMap;

use crate::{parse_msg::match_lines, DataType, FieldInfo, MessagePath, Result};

#[derive(Clone, Debug)]
pub struct Msg {
    pub path: MessagePath,
    pub fields: Vec<FieldInfo>,
    pub source: String,
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

    pub fn get_type(&self) -> String {
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
