use crate::{Error, MessagePath, Msg, Result};
use regex::RegexBuilder;

#[derive(Clone, Debug)]
pub struct Srv {
    pub path: MessagePath,
    pub source: String,
}

#[derive(Clone, Debug)]
pub struct SrvMessages {
    pub req: Msg,
    pub res: Msg,
}

impl Srv {
    pub fn new(path: MessagePath, source: impl Into<String>) -> Srv {
        Srv {
            path,
            source: source.into(),
        }
    }

    pub fn build_messages(&self) -> Result<SrvMessages> {
        let re = RegexBuilder::new("^---$")
            .multi_line(true)
            .build()
            .expect("Invalid regex `^---$`");
        let (req, res) = match re.split(&self.source).collect::<Vec<_>>().as_slice() {
            &[req] => (req, ""),
            &[req, res] => (req, res),
            &[] => {
                return Err(Error::BadMessageContent(format!(
                    "Service {} does not have any content",
                    self.path
                )))
            }
            v => {
                return Err(Error::BadMessageContent(format!(
                    "Service {} is split into {} parts",
                    self.path,
                    v.len()
                )))
            }
        };

        Ok(SrvMessages {
            req: Msg::new(
                MessagePath::new(self.path.package(), format!("{}Req", self.path.name()))?,
                req,
            )?,
            res: Msg::new(
                MessagePath::new(self.path.package(), format!("{}Res", self.path.name()))?,
                res,
            )?,
        })
    }
}
