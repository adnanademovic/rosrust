use crate::{Error, Result};
use lazy_static::lazy_static;
use regex::Regex;
use serde_derive::{Deserialize, Serialize};
use std::convert::TryFrom;
use std::fmt::{Display, Formatter};
use std::hash::Hash;

#[derive(Clone, Debug, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct MessagePath {
    package: String,
    name: String,
}

pub fn is_valid_package_name(package: &str) -> bool {
    lazy_static! {
        static ref RE_PACKAGE_CORRECT_CHAR_SET_AND_LENGTH: Regex =
            Regex::new("^[a-z][a-z0-9_]+$").unwrap();
        static ref RE_PACKAGE_CONSECUTIVE_UNDERSCORE: Regex = Regex::new("__").unwrap();
    }
    RE_PACKAGE_CORRECT_CHAR_SET_AND_LENGTH.is_match(package)
        && !RE_PACKAGE_CONSECUTIVE_UNDERSCORE.is_match(package)
}

impl MessagePath {
    /// Create full message path, with naming rules checked
    ///
    /// Naming rules are based on [REP 144](https://www.ros.org/reps/rep-0144.html).
    pub fn new(package: impl Into<String>, name: impl Into<String>) -> Result<Self> {
        let package = package.into();
        let name = name.into();
        if !is_valid_package_name(&package) {
            return Err(Error::InvalidMessagePath  {
                name: format!("{}/{}",package,name),
                  reason: "package name needs to follow REP 144 rules (https://www.ros.org/reps/rep-0144.html)".into(),
            });
        }
        Ok(Self { package, name })
    }

    fn from_combined(input: &str) -> Result<Self> {
        let parts = input.splitn(2, '/').collect::<Vec<&str>>();
        match parts[..] {
            [package, name] => Self::new(package, name),
            _ => Err(Error::InvalidMessagePath {
                name: input.into(),
                reason: "string needs to be in `package/name` format".into(),
            }),
        }
    }

    pub fn package(&self) -> &str {
        &self.package
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn into_inner(self) -> (String, String) {
        (self.package, self.name)
    }
}

impl Display for MessagePath {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}/{}", self.package(), self.name())
    }
}

impl<'a> TryFrom<&'a str> for MessagePath {
    type Error = Error;

    fn try_from(value: &'a str) -> Result<Self> {
        Self::from_combined(value)
    }
}
