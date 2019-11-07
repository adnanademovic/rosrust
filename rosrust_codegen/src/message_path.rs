use crate::error::Result;
use error_chain::bail;
use std::hash::{Hash, Hasher};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MessagePath {
    pub package: String,
    pub name: String,
}

impl MessagePath {
    pub fn new(package: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            package: package.into(),
            name: name.into(),
        }
    }

    pub fn from_combined(input: &str) -> Result<Self> {
        let mut parts = input.splitn(2, '/');
        let package = match parts.next() {
            Some(v) => v,
            None => bail!("Package string constains no parts: {}", input),
        };
        let name = match parts.next() {
            Some(v) => v,
            None => bail!(
                "Package string needs to be in package/name format: {}",
                input
            ),
        };
        Ok(Self::new(package, name))
    }
}

impl Hash for MessagePath {
    fn hash<H: Hasher>(&self, state: &mut H) {
        (&self.package, &self.name).hash(state)
    }
}
