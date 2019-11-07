use crate::error::{Error, ErrorKind, Result};
use error_chain::bail;
use lazy_static::lazy_static;
use regex::Regex;
use std::convert::TryFrom;
use std::fmt::{Display, Formatter};
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

    fn from_combined(input: &str) -> Result<Self> {
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
        let output = Self::new(package, name);
        output.validate()?;
        Ok(output)
    }

    /// Perform package name validity checks
    ///
    /// Based on [REP 144](https://www.ros.org/reps/rep-0144.html).
    fn has_valid_package_name(&self) -> bool {
        RE_PACKAGE_CORRECT_CHAR_SET_AND_LENGTH.is_match(&self.package)
            && !RE_PACKAGE_CONSECUTIVE_UNDERSCORE.is_match(&self.package)
    }

    pub fn validate(&self) -> Result<()> {
        if !self.has_valid_package_name() {
            bail!(ErrorKind::PackageNameInvalid(self.package.clone()));
        }
        Ok(())
    }
}

impl Display for MessagePath {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}/{}", self.package, self.name)
    }
}

impl<'a> TryFrom<&'a str> for MessagePath {
    type Error = Error;

    fn try_from(value: &'a str) -> Result<Self> {
        Self::from_combined(value)
    }
}

impl Hash for MessagePath {
    fn hash<H: Hasher>(&self, state: &mut H) {
        (&self.package, &self.name).hash(state)
    }
}

lazy_static! {
    static ref RE_PACKAGE_CORRECT_CHAR_SET_AND_LENGTH: Regex =
        Regex::new("^[a-z][a-z0-9_]+$").unwrap();
    static ref RE_PACKAGE_CONSECUTIVE_UNDERSCORE: Regex = Regex::new("__").unwrap();
}

#[cfg(test)]
mod tests {
    use super::*;

    fn is_valid_package_name(package: &str) -> bool {
        MessagePath::new(package, "anything").has_valid_package_name()
    }

    #[test]
    fn package_names_must_be_at_least_two_characters() {
        assert!(is_valid_package_name("foo"));
        assert!(is_valid_package_name("fo"));
        assert!(!is_valid_package_name("f"));
    }

    #[test]
    fn package_names_must_start_with_lowercase_alphabetic() {
        assert!(is_valid_package_name("foo_123"));
        assert!(!is_valid_package_name("Foo_123"));
        assert!(!is_valid_package_name("1oo_123"));
        assert!(!is_valid_package_name("_oo_123"));
    }

    #[test]
    fn package_names_must_not_contain_uppercase_anywhere() {
        assert!(is_valid_package_name("foo_123"));
        assert!(!is_valid_package_name("foO_123"));
    }

    #[test]
    fn package_names_are_limited_to_lowercase_alphanumeric_and_underscore() {
        assert!(is_valid_package_name("foo_123"));
        assert!(!is_valid_package_name("foO_123"));
        assert!(!is_valid_package_name("foo-123"));
    }

    #[test]
    fn package_names_must_not_contain_multiple_underscores_in_a_row() {
        assert!(is_valid_package_name("foo_123_"));
        assert!(is_valid_package_name("f_o_o_1_2_3_"));
        assert!(!is_valid_package_name("foo__123_"));
        assert!(!is_valid_package_name("foo___123_"));
    }
}
