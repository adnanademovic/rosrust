use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use std;
use std::io;
use time::{Duration, Time};

pub trait RosMsg: std::marker::Sized {
    fn encode<W: io::Write>(&self, w: &mut W) -> io::Result<()>;
    fn decode<R: io::Read>(r: &mut R) -> io::Result<Self>;

    #[inline]
    fn encode_vec(&self) -> io::Result<Vec<u8>> {
        let mut writer = Vec::with_capacity(128);
        self.encode(&mut writer)?;
        Ok(writer)
    }

    #[inline]
    fn decode_slice(bytes: &[u8]) -> io::Result<Self> {
        Self::decode(&mut io::Cursor::new(bytes))
    }
}

impl RosMsg for u8 {
    #[inline]
    fn encode<W: io::Write>(&self, w: &mut W) -> io::Result<()> {
        w.write_u8(*self)
    }

    #[inline]
    fn decode<R: io::Read>(r: &mut R) -> io::Result<Self> {
        r.read_u8()
    }
}

impl RosMsg for i8 {
    #[inline]
    fn encode<W: io::Write>(&self, w: &mut W) -> io::Result<()> {
        w.write_i8(*self)
    }

    #[inline]
    fn decode<R: io::Read>(r: &mut R) -> io::Result<Self> {
        r.read_i8()
    }
}

impl RosMsg for u16 {
    #[inline]
    fn encode<W: io::Write>(&self, w: &mut W) -> io::Result<()> {
        w.write_u16::<LittleEndian>(*self)
    }

    #[inline]
    fn decode<R: io::Read>(r: &mut R) -> io::Result<Self> {
        r.read_u16::<LittleEndian>()
    }
}

impl RosMsg for i16 {
    #[inline]
    fn encode<W: io::Write>(&self, w: &mut W) -> io::Result<()> {
        w.write_i16::<LittleEndian>(*self)
    }

    #[inline]
    fn decode<R: io::Read>(r: &mut R) -> io::Result<Self> {
        r.read_i16::<LittleEndian>()
    }
}

impl RosMsg for u32 {
    #[inline]
    fn encode<W: io::Write>(&self, w: &mut W) -> io::Result<()> {
        w.write_u32::<LittleEndian>(*self)
    }

    #[inline]
    fn decode<R: io::Read>(r: &mut R) -> io::Result<Self> {
        r.read_u32::<LittleEndian>()
    }
}

impl RosMsg for i32 {
    #[inline]
    fn encode<W: io::Write>(&self, w: &mut W) -> io::Result<()> {
        w.write_i32::<LittleEndian>(*self)
    }

    #[inline]
    fn decode<R: io::Read>(r: &mut R) -> io::Result<Self> {
        r.read_i32::<LittleEndian>()
    }
}

impl RosMsg for u64 {
    #[inline]
    fn encode<W: io::Write>(&self, w: &mut W) -> io::Result<()> {
        w.write_u64::<LittleEndian>(*self)
    }

    #[inline]
    fn decode<R: io::Read>(r: &mut R) -> io::Result<Self> {
        r.read_u64::<LittleEndian>()
    }
}

impl RosMsg for i64 {
    #[inline]
    fn encode<W: io::Write>(&self, w: &mut W) -> io::Result<()> {
        w.write_i64::<LittleEndian>(*self)
    }

    #[inline]
    fn decode<R: io::Read>(r: &mut R) -> io::Result<Self> {
        r.read_i64::<LittleEndian>()
    }
}

impl RosMsg for f32 {
    #[inline]
    fn encode<W: io::Write>(&self, w: &mut W) -> io::Result<()> {
        w.write_f32::<LittleEndian>(*self)
    }

    #[inline]
    fn decode<R: io::Read>(r: &mut R) -> io::Result<Self> {
        r.read_f32::<LittleEndian>()
    }
}

impl RosMsg for f64 {
    #[inline]
    fn encode<W: io::Write>(&self, w: &mut W) -> io::Result<()> {
        w.write_f64::<LittleEndian>(*self)
    }

    #[inline]
    fn decode<R: io::Read>(r: &mut R) -> io::Result<Self> {
        r.read_f64::<LittleEndian>()
    }
}

#[inline]
pub fn encode_fixed_slice<W: io::Write, T: RosMsg>(data: &[T], w: &mut W) -> io::Result<()> {
    data.into_iter().try_for_each(|v| v.encode(w))
}

#[inline]
pub fn decode_fixed_vec<R: io::Read, T: RosMsg>(len: u32, r: &mut R) -> io::Result<Vec<T>> {
    (0..len).map(|_| T::decode(r)).collect()
}

#[inline]
pub fn encode_variable_slice<W: io::Write, T: RosMsg>(data: &[T], w: &mut W) -> io::Result<()> {
    (data.len() as u32).encode(w)?;
    encode_fixed_slice(data, w)
}

#[inline]
pub fn decode_variable_vec<R: io::Read, T: RosMsg>(r: &mut R) -> io::Result<Vec<T>> {
    decode_fixed_vec(u32::decode(r)?, r)
}

#[inline]
pub fn encode_str<W: io::Write>(value: &str, w: &mut W) -> io::Result<()> {
    encode_variable_slice(value.as_bytes(), w)
}

impl RosMsg for String {
    #[inline]
    fn encode<W: io::Write>(&self, w: &mut W) -> io::Result<()> {
        encode_str(self, w)
    }

    #[inline]
    fn decode<R: io::Read>(r: &mut R) -> io::Result<Self> {
        decode_variable_vec::<R, u8>(r).and_then(|v| {
            String::from_utf8(v).map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err))
        })
    }
}

impl RosMsg for Time {
    #[inline]
    fn encode<W: io::Write>(&self, w: &mut W) -> io::Result<()> {
        self.sec.encode(w)?;
        self.nsec.encode(w)?;
        Ok(())
    }

    #[inline]
    fn decode<R: io::Read>(r: &mut R) -> io::Result<Self> {
        Ok(Self {
            sec: RosMsg::decode(r)?,
            nsec: RosMsg::decode(r)?,
        })
    }
}

impl RosMsg for Duration {
    #[inline]
    fn encode<W: io::Write>(&self, w: &mut W) -> io::Result<()> {
        self.sec.encode(w)?;
        self.nsec.encode(w)?;
        Ok(())
    }

    #[inline]
    fn decode<R: io::Read>(r: &mut R) -> io::Result<Self> {
        Ok(Self {
            sec: RosMsg::decode(r)?,
            nsec: RosMsg::decode(r)?,
        })
    }
}
