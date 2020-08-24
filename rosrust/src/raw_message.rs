use crate::{Message, RosMsg};
use std::io;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct RawMessage(pub Vec<u8>);

#[derive(Clone, Debug, PartialEq)]
pub struct RawMessageDescription {
    pub msg_definition: String,
    pub md5sum: String,
    pub msg_type: String,
}

impl RawMessageDescription {
    pub fn from_message<T: Message>() -> Self {
        Self {
            msg_definition: T::msg_definition(),
            md5sum: T::md5sum(),
            msg_type: T::msg_type(),
        }
    }
}

impl Message for RawMessage {
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

impl RosMsg for RawMessage {
    fn encode<W: io::Write>(&self, mut w: W) -> io::Result<()> {
        w.write_all(&self.0)
    }

    fn decode<R: io::Read>(mut r: R) -> io::Result<Self> {
        let mut data = vec![];
        r.read_to_end(&mut data)?;
        Ok(Self(data))
    }
}
