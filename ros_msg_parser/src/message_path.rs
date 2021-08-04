use crate::{Error, Result};
use lazy_static::lazy_static;
use regex::Regex;
use std::convert::TryFrom;
use std::fmt::{Display, Formatter};
use std::hash::Hash;

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct MessagePath {
    package: String,
    name: String,
}

fn is_valid_package_name(package: &str) -> bool {
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
                reason: "string needs to be in package/name format".into(),
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

#[cfg(test)]
mod tests {
    use super::*;

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
