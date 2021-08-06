use crate::{Error, MessagePath, Msg, Result};
use lazy_static::lazy_static;
use regex::RegexBuilder;
use serde_derive::{Deserialize, Serialize};
use std::fmt;
use std::fmt::Formatter;

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

    pub fn into_inner(self) -> (MessagePath, String, Msg, Msg) {
        (self.path, self.source, self.req, self.res)
    }

    pub fn path(&self) -> &MessagePath {
        &self.path
    }

    pub fn source(&self) -> &str {
        &self.source
    }

    pub fn req(&self) -> &Msg {
        &self.req
    }

    pub fn res(&self) -> &Msg {
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
            Msg::new(
                MessagePath::new(path.package(), format!("{}Req", path.name()))?,
                req,
            )?,
            Msg::new(
                MessagePath::new(path.package(), format!("{}Res", path.name()))?,
                res,
            )?,
        ))
    }
}
