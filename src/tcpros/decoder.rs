use byteorder::{LittleEndian, ReadBytesExt};
use rustc_serialize;
use std;
use super::error::Error;

pub struct Decoder {
    input: std::vec::IntoIter<u8>,
    bytes_read: usize,
}

impl Decoder {
    pub fn new(data: Vec<u8>) -> Decoder {
        Decoder {
            input: data.into_iter(),
            bytes_read: 0,
        }
    }

    fn pop_length(&mut self) -> Result<u32, Error> {
        let mut reader = std::io::Cursor::new(try!(self.read_bytes(4)));
        Ok(try!(reader.read_u32::<LittleEndian>()))
    }

    fn read_bytes(&mut self, count: u32) -> Result<Vec<u8>, Error> {
        let mut buffer = vec![];
        for _ in 0..count {
            buffer.push(try!(self.input.next().ok_or(Error::Truncated)));
            self.bytes_read += 1;
        }
        Ok(buffer)
    }
}

macro_rules! match_length {
    ($s:expr, $x:expr) => (if $x != try!($s.pop_length()) {return Err(Error::Mismatch)});
}

impl rustc_serialize::Decoder for Decoder {
    type Error = Error;

    fn read_nil(&mut self) -> Result<(), Self::Error> {
        Err(Error::UnsupportedData)
    }

    fn read_usize(&mut self) -> Result<usize, Self::Error> {
        Err(Error::UnsupportedData)
    }

    fn read_u64(&mut self) -> Result<u64, Self::Error> {
        match_length!(self, 8);
        let mut reader = std::io::Cursor::new(try!(self.read_bytes(8)));
        Ok(try!(reader.read_u64::<LittleEndian>()))
    }

    fn read_u32(&mut self) -> Result<u32, Self::Error> {
        match_length!(self, 4);
        let mut reader = std::io::Cursor::new(try!(self.read_bytes(4)));
        Ok(try!(reader.read_u32::<LittleEndian>()))
    }

    fn read_u16(&mut self) -> Result<u16, Self::Error> {
        match_length!(self, 2);
        let mut reader = std::io::Cursor::new(try!(self.read_bytes(2)));
        Ok(try!(reader.read_u16::<LittleEndian>()))
    }

    fn read_u8(&mut self) -> Result<u8, Self::Error> {
        match_length!(self, 1);
        let mut reader = std::io::Cursor::new(try!(self.read_bytes(1)));
        Ok(try!(reader.read_u8()))
    }

    fn read_isize(&mut self) -> Result<isize, Self::Error> {
        Err(Error::UnsupportedData)
    }

    fn read_i64(&mut self) -> Result<i64, Self::Error> {
        match_length!(self, 8);
        let mut reader = std::io::Cursor::new(try!(self.read_bytes(8)));
        Ok(try!(reader.read_i64::<LittleEndian>()))
    }

    fn read_i32(&mut self) -> Result<i32, Self::Error> {
        match_length!(self, 4);
        let mut reader = std::io::Cursor::new(try!(self.read_bytes(4)));
        Ok(try!(reader.read_i32::<LittleEndian>()))
    }

    fn read_i16(&mut self) -> Result<i16, Self::Error> {
        match_length!(self, 2);
        let mut reader = std::io::Cursor::new(try!(self.read_bytes(2)));
        Ok(try!(reader.read_i16::<LittleEndian>()))
    }

    fn read_i8(&mut self) -> Result<i8, Self::Error> {
        match_length!(self, 1);
        let mut reader = std::io::Cursor::new(try!(self.read_bytes(1)));
        Ok(try!(reader.read_i8()))
    }

    fn read_bool(&mut self) -> Result<bool, Self::Error> {
        match_length!(self, 1);
        let mut reader = std::io::Cursor::new(try!(self.read_bytes(1)));
        Ok(try!(reader.read_u8()) != 0)
    }

    fn read_f64(&mut self) -> Result<f64, Self::Error> {
        match_length!(self, 8);
        let mut reader = std::io::Cursor::new(try!(self.read_bytes(8)));
        Ok(try!(reader.read_f64::<LittleEndian>()))
    }

    fn read_f32(&mut self) -> Result<f32, Self::Error> {
        match_length!(self, 4);
        let mut reader = std::io::Cursor::new(try!(self.read_bytes(4)));
        Ok(try!(reader.read_f32::<LittleEndian>()))
    }

    fn read_char(&mut self) -> Result<char, Self::Error> {
        Err(Error::UnsupportedData)
    }

    fn read_str(&mut self) -> Result<String, Self::Error> {
        let length = try!(self.pop_length());
        String::from_utf8(try!(self.read_bytes(length))).or(Err(Error::UnsupportedData))
    }

    fn read_enum<T, F>(&mut self, _: &str, _: F) -> Result<T, Self::Error>
        where F: FnOnce(&mut Self) -> Result<T, Self::Error>
    {
        Err(Error::UnsupportedData)
    }

    fn read_enum_variant<T, F>(&mut self, _: &[&str], _: F) -> Result<T, Self::Error>
        where F: FnMut(&mut Self, usize) -> Result<T, Self::Error>
    {
        Err(Error::UnsupportedData)
    }

    fn read_enum_variant_arg<T, F>(&mut self, _: usize, _: F) -> Result<T, Self::Error>
        where F: FnOnce(&mut Self) -> Result<T, Self::Error>
    {
        Err(Error::UnsupportedData)
    }

    fn read_enum_struct_variant<T, F>(&mut self, _: &[&str], _: F) -> Result<T, Self::Error>
        where F: FnMut(&mut Self, usize) -> Result<T, Self::Error>
    {
        Err(Error::UnsupportedData)
    }

    fn read_enum_struct_variant_field<T, F>(&mut self,
                                            _: &str,
                                            _: usize,
                                            _: F)
                                            -> Result<T, Self::Error>
        where F: FnOnce(&mut Self) -> Result<T, Self::Error>
    {
        Err(Error::UnsupportedData)
    }

    fn read_struct<T, F>(&mut self, _: &str, _: usize, f: F) -> Result<T, Self::Error>
        where F: FnOnce(&mut Self) -> Result<T, Self::Error>
    {
        let length = try!(self.pop_length());
        let data_start = self.bytes_read;
        let retval = f(self);
        if self.bytes_read == data_start + length as usize {
            retval
        } else {
            Err(Error::Truncated)
        }
    }

    fn read_struct_field<T, F>(&mut self, _: &str, _: usize, f: F) -> Result<T, Self::Error>
        where F: FnOnce(&mut Self) -> Result<T, Self::Error>
    {
        f(self)
    }

    fn read_tuple<T, F>(&mut self, _: usize, f: F) -> Result<T, Self::Error>
        where F: FnOnce(&mut Self) -> Result<T, Self::Error>
    {
        let length = try!(self.pop_length());
        let data_start = self.bytes_read;
        let retval = f(self);
        if self.bytes_read == data_start + length as usize {
            retval
        } else {
            Err(Error::Truncated)
        }
    }

    fn read_tuple_arg<T, F>(&mut self, _: usize, f: F) -> Result<T, Self::Error>
        where F: FnOnce(&mut Self) -> Result<T, Self::Error>
    {
        f(self)
    }

    fn read_tuple_struct<T, F>(&mut self, _: &str, _: usize, _: F) -> Result<T, Self::Error>
        where F: FnOnce(&mut Self) -> Result<T, Self::Error>
    {
        Err(Error::UnsupportedData)
    }

    fn read_tuple_struct_arg<T, F>(&mut self, _: usize, _: F) -> Result<T, Self::Error>
        where F: FnOnce(&mut Self) -> Result<T, Self::Error>
    {
        Err(Error::UnsupportedData)
    }

    fn read_option<T, F>(&mut self, _: F) -> Result<T, Self::Error>
        where F: FnMut(&mut Self, bool) -> Result<T, Self::Error>
    {
        Err(Error::UnsupportedData)
    }

    fn read_seq<T, F>(&mut self, f: F) -> Result<T, Self::Error>
        where F: FnOnce(&mut Self, usize) -> Result<T, Self::Error>
    {
        let length = try!(self.pop_length());
        let data_start = self.bytes_read;
        let count = try!(self.pop_length());
        let retval = f(self, count as usize);
        if self.bytes_read == data_start + length as usize {
            retval
        } else {
            Err(Error::Truncated)
        }
    }

    fn read_seq_elt<T, F>(&mut self, _: usize, f: F) -> Result<T, Self::Error>
        where F: FnOnce(&mut Self) -> Result<T, Self::Error>
    {
        f(self)
    }

    fn read_map<T, F>(&mut self, _: F) -> Result<T, Self::Error>
        where F: FnOnce(&mut Self, usize) -> Result<T, Self::Error>
    {
        Err(Error::UnsupportedData)
    }

    fn read_map_elt_key<T, F>(&mut self, _: usize, _: F) -> Result<T, Self::Error>
        where F: FnOnce(&mut Self) -> Result<T, Self::Error>
    {
        Err(Error::UnsupportedData)
    }

    fn read_map_elt_val<T, F>(&mut self, _: usize, _: F) -> Result<T, Self::Error>
        where F: FnOnce(&mut Self) -> Result<T, Self::Error>
    {
        Err(Error::UnsupportedData)
    }

    fn error(&mut self, err: &str) -> Self::Error {
        Error::Other(err.to_owned())
    }
}
