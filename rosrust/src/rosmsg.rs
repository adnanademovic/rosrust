use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use std;
use std::collections::HashMap;
use std::io;
use time::{Duration, Time};

pub trait RosMsg: std::marker::Sized {
    fn encode<W: io::Write>(&self, w: W) -> io::Result<()>;
    fn decode<R: io::Read>(r: R) -> io::Result<Self>;

    #[inline]
    fn encode_vec(&self) -> io::Result<Vec<u8>> {
        let mut writer = io::Cursor::new(Vec::with_capacity(128));
        // skip the first 4 bytes that will contain the message length
        writer.set_position(4);

        self.encode(&mut writer)?;

        // write the message length to the start of the header
        let message_length = (writer.position() - 4) as u32;
        writer.set_position(0);
        message_length.encode(&mut writer)?;
        Ok(writer.into_inner())
    }

    #[inline]
    fn decode_slice(bytes: &[u8]) -> io::Result<Self> {
        let mut reader = io::Cursor::new(bytes);
        // skip the first 4 bytes that contain the message length
        reader.set_position(4);
        Self::decode(&mut reader)
    }
}

impl RosMsg for u8 {
    #[inline]
    fn encode<W: io::Write>(&self, mut w: W) -> io::Result<()> {
        w.write_u8(*self)
    }

    #[inline]
    fn decode<R: io::Read>(mut r: R) -> io::Result<Self> {
        r.read_u8()
    }
}

impl RosMsg for i8 {
    #[inline]
    fn encode<W: io::Write>(&self, mut w: W) -> io::Result<()> {
        w.write_i8(*self)
    }

    #[inline]
    fn decode<R: io::Read>(mut r: R) -> io::Result<Self> {
        r.read_i8()
    }
}

impl RosMsg for u16 {
    #[inline]
    fn encode<W: io::Write>(&self, mut w: W) -> io::Result<()> {
        w.write_u16::<LittleEndian>(*self)
    }

    #[inline]
    fn decode<R: io::Read>(mut r: R) -> io::Result<Self> {
        r.read_u16::<LittleEndian>()
    }
}

impl RosMsg for i16 {
    #[inline]
    fn encode<W: io::Write>(&self, mut w: W) -> io::Result<()> {
        w.write_i16::<LittleEndian>(*self)
    }

    #[inline]
    fn decode<R: io::Read>(mut r: R) -> io::Result<Self> {
        r.read_i16::<LittleEndian>()
    }
}

impl RosMsg for u32 {
    #[inline]
    fn encode<W: io::Write>(&self, mut w: W) -> io::Result<()> {
        w.write_u32::<LittleEndian>(*self)
    }

    #[inline]
    fn decode<R: io::Read>(mut r: R) -> io::Result<Self> {
        r.read_u32::<LittleEndian>()
    }
}

impl RosMsg for i32 {
    #[inline]
    fn encode<W: io::Write>(&self, mut w: W) -> io::Result<()> {
        w.write_i32::<LittleEndian>(*self)
    }

    #[inline]
    fn decode<R: io::Read>(mut r: R) -> io::Result<Self> {
        r.read_i32::<LittleEndian>()
    }
}

impl RosMsg for u64 {
    #[inline]
    fn encode<W: io::Write>(&self, mut w: W) -> io::Result<()> {
        w.write_u64::<LittleEndian>(*self)
    }

    #[inline]
    fn decode<R: io::Read>(mut r: R) -> io::Result<Self> {
        r.read_u64::<LittleEndian>()
    }
}

impl RosMsg for i64 {
    #[inline]
    fn encode<W: io::Write>(&self, mut w: W) -> io::Result<()> {
        w.write_i64::<LittleEndian>(*self)
    }

    #[inline]
    fn decode<R: io::Read>(mut r: R) -> io::Result<Self> {
        r.read_i64::<LittleEndian>()
    }
}

impl RosMsg for f32 {
    #[inline]
    fn encode<W: io::Write>(&self, mut w: W) -> io::Result<()> {
        w.write_f32::<LittleEndian>(*self)
    }

    #[inline]
    fn decode<R: io::Read>(mut r: R) -> io::Result<Self> {
        r.read_f32::<LittleEndian>()
    }
}

impl RosMsg for f64 {
    #[inline]
    fn encode<W: io::Write>(&self, mut w: W) -> io::Result<()> {
        w.write_f64::<LittleEndian>(*self)
    }

    #[inline]
    fn decode<R: io::Read>(mut r: R) -> io::Result<Self> {
        r.read_f64::<LittleEndian>()
    }
}

#[inline]
pub fn encode_fixed_slice<W: io::Write, T: RosMsg>(data: &[T], mut w: W) -> io::Result<()> {
    data.into_iter().try_for_each(|v| v.encode(w.by_ref()))
}

#[inline]
pub fn decode_fixed_vec<R: io::Read, T: RosMsg>(len: u32, mut r: R) -> io::Result<Vec<T>> {
    (0..len).map(move |_| T::decode(r.by_ref())).collect()
}

#[inline]
pub fn encode_variable_slice<W: io::Write, T: RosMsg>(data: &[T], mut w: W) -> io::Result<()> {
    (data.len() as u32).encode(w.by_ref())?;
    encode_fixed_slice(data, w)
}

#[inline]
pub fn decode_variable_vec<R: io::Read, T: RosMsg>(mut r: R) -> io::Result<Vec<T>> {
    decode_fixed_vec(u32::decode(r.by_ref())?, r)
}

#[inline]
pub fn encode_str<W: io::Write>(value: &str, w: W) -> io::Result<()> {
    encode_variable_slice(value.as_bytes(), w)
}

impl RosMsg for String {
    #[inline]
    fn encode<W: io::Write>(&self, w: W) -> io::Result<()> {
        encode_str(self, w)
    }

    #[inline]
    fn decode<R: io::Read>(r: R) -> io::Result<Self> {
        decode_variable_vec::<R, u8>(r).and_then(|v| {
            String::from_utf8(v).map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err))
        })
    }
}

impl RosMsg for HashMap<String, String> {
    #[inline]
    fn encode<W: io::Write>(&self, mut w: W) -> io::Result<()> {
        let rows = self
            .iter()
            .map(|(key, value)| format!("{}={}", key, value))
            .collect::<Vec<String>>();
        let data_size: usize = rows.iter().map(|item| item.len() + 4).sum();
        write_data_size(data_size as u32, w.by_ref())?;
        rows.into_iter()
            .try_for_each(|item| item.encode(w.by_ref()))
    }

    #[inline]
    fn decode<R: io::Read>(mut r: R) -> io::Result<Self> {
        let data_size = read_data_size(r.by_ref())? as u64;
        let mut limited_r = r.take(data_size);
        let mut output = HashMap::<String, String>::default();
        loop {
            let item = match String::decode(&mut limited_r) {
                Ok(val) => val,
                Err(_err) => break, // TODO: ensure we break only on EOF
            };
            match item.splitn(2, "=").collect::<Vec<&str>>().as_slice() {
                &[key, value] => output.insert(key.into(), value.into()),
                _ => {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        "Map rows need to have a format of key=value",
                    ))
                }
            };
        }
        Ok(output)
    }
}

impl RosMsg for Time {
    #[inline]
    fn encode<W: io::Write>(&self, mut w: W) -> io::Result<()> {
        self.sec.encode(w.by_ref())?;
        self.nsec.encode(w)?;
        Ok(())
    }

    #[inline]
    fn decode<R: io::Read>(mut r: R) -> io::Result<Self> {
        Ok(Self {
            sec: RosMsg::decode(r.by_ref())?,
            nsec: RosMsg::decode(r)?,
        })
    }
}

impl RosMsg for Duration {
    #[inline]
    fn encode<W: io::Write>(&self, mut w: W) -> io::Result<()> {
        self.sec.encode(w.by_ref())?;
        self.nsec.encode(w)?;
        Ok(())
    }

    #[inline]
    fn decode<R: io::Read>(mut r: R) -> io::Result<Self> {
        Ok(Self {
            sec: RosMsg::decode(r.by_ref())?,
            nsec: RosMsg::decode(r)?,
        })
    }
}

#[inline]
fn read_data_size<R: io::Read>(r: R) -> io::Result<u32> {
    u32::decode(r)
}

#[inline]
fn write_data_size<W: io::Write>(value: u32, w: W) -> io::Result<()> {
    value.encode(w)
}
