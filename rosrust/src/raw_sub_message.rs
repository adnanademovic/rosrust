use crate::{Message, RosMsg};
use std::io;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct RawSubMessage(pub Vec<u8>);

impl Message for RawSubMessage {
    fn msg_definition() -> String {
        "*".into()
    }

    fn md5sum() -> String {
        "*".into()
    }

    fn msg_type() -> String {
        "*".into()
    }
}

impl RosMsg for RawSubMessage {
    fn encode<W: io::Write>(&self, mut w: W) -> io::Result<()> {
        w.write_all(&self.0)
    }

    fn decode<R: io::Read>(mut r: R) -> io::Result<Self> {
        let mut data = vec![];
        r.read_to_end(&mut data)?;
        Ok(Self(data))
    }
}
