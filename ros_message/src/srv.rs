use crate::{Error, MessagePath, Msg, Result};
use lazy_static::lazy_static;
use regex::RegexBuilder;
use serde_derive::{Deserialize, Serialize};
use std::fmt;
use std::fmt::Formatter;

/// A ROS service parsed from a `srv` file.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Srv {
    path: MessagePath,
    source: String,
    req: Msg,
    res: Msg,
}

impl fmt::Display for Srv {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.source.fmt(f)
    }
}

impl Srv {
    /// Create a service from a passed in path and source.
    ///
    /// # Errors
    ///
    /// Returns an error if there is an error parsing the service source.
    ///
    /// # Examples
    ///
    /// ```
    /// # use ros_message::Srv;
    /// # use std::convert::TryInto;
    /// #
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let service = Srv::new(
    ///     "foo/Bar".try_into()?,
    ///     r#"# a comment that is ignored
    ///     Header header
    ///     uint32 a
    ///     byte[16] b
    ///     geometry_msgs/Point[] point
    ///     uint32 FOO=5
    ///     string SOME_TEXT=this is # some text, don't be fooled by the hash
    /// ---
    ///     uint32 a
    ///     geometry_msgs/Point[] point
    ///     uint32 FOO=6
    ///     "#,
    /// )?;
    ///
    /// assert_eq!(service.path(), &"foo/Bar".try_into()?);
    /// assert_eq!(service.request().fields().len(), 6);
    /// assert_eq!(service.response().fields().len(), 3);
    /// # Ok(())
    /// # }
    /// ```
    pub fn new(path: MessagePath, source: impl Into<String>) -> Result<Srv> {
        let source = source.into();
        let (req, res) = Self::build_req_res(&path, &source)?;
        Ok(Srv {
            path,
            source,
            req,
            res,
        })
    }

    /// Returns the path of the service.
    pub fn path(&self) -> &MessagePath {
        &self.path
    }

    /// Returns the original source.
    pub fn source(&self) -> &str {
        &self.source
    }

    /// Returns the request message.
    pub fn request(&self) -> &Msg {
        &self.req
    }

    /// Returns the response message.
    pub fn response(&self) -> &Msg {
        &self.res
    }

    fn build_req_res(path: &MessagePath, source: &str) -> Result<(Msg, Msg)> {
        lazy_static! {
            static ref RE_SPLIT: regex::Regex = RegexBuilder::new("^---$")
                .multi_line(true)
                .build()
                .expect("Invalid regex `^---$`");
        }
        let (req, res) = match RE_SPLIT.split(source).collect::<Vec<_>>().as_slice() {
            &[req] => (req, ""),
            &[req, res] => (req, res),
            &[] => {
                return Err(Error::BadMessageContent(format!(
                    "Service {} does not have any content",
                    path
                )))
            }
            v => {
                return Err(Error::BadMessageContent(format!(
                    "Service {} is split into {} parts",
                    path,
                    v.len()
                )))
            }
        };

        Ok((
            Msg::new(path.peer(format!("{}Req", path.name())), req)?,
            Msg::new(path.peer(format!("{}Res", path.name())), res)?,
        ))
    }
}
