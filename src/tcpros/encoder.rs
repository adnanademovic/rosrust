use byteorder::{LittleEndian, WriteBytesExt};
use rustc_serialize;
use super::error::Error;

#[derive(Debug)]
pub struct Encoder {
    output: Vec<u8>,
}

impl Encoder {
    pub fn new() -> Encoder {
        Encoder { output: Vec::<u8>::new() }
    }

    pub fn extract_data(self) -> Vec<u8> {
        self.output
    }

    fn write_size(&mut self, v: usize) -> Result<(), Error> {
        let v = v as u32;
        let mut buffer = vec![];
        try!(buffer.write_u32::<LittleEndian>(v));
        for &byte in buffer.iter() {
            self.output.push(byte);
        }
        Ok(())
    }

    fn write_size_in_middle(&mut self, position: usize, v: usize) -> Result<(), Error> {
        let v = v as u32;
        let mut buffer = vec![];
        try!(buffer.write_u32::<LittleEndian>(v));
        let mut i = 0;
        for &byte in buffer.iter() {
            unsafe { *self.output.get_unchecked_mut(position + i) = byte };
            i += 1;
        }
        Ok(())
    }
}

impl rustc_serialize::Encoder for Encoder {
    type Error = Error;

    fn emit_nil(&mut self) -> Result<(), Self::Error> {
        Err(Error::UnsupportedData)
    }

    fn emit_usize(&mut self, _: usize) -> Result<(), Self::Error> {
        Err(Error::UnsupportedData)
    }

    fn emit_u64(&mut self, v: u64) -> Result<(), Self::Error> {
        try!(self.write_size(8));
        let mut buffer = vec![];
        try!(buffer.write_u64::<LittleEndian>(v));
        for &byte in buffer.iter() {
            self.output.push(byte);
        }
        Ok(())
    }

    fn emit_u32(&mut self, v: u32) -> Result<(), Self::Error> {
        try!(self.write_size(4));
        let mut buffer = vec![];
        try!(buffer.write_u32::<LittleEndian>(v));
        for &byte in buffer.iter() {
            self.output.push(byte);
        }
        Ok(())
    }

    fn emit_u16(&mut self, v: u16) -> Result<(), Self::Error> {
        try!(self.write_size(2));
        let mut buffer = vec![];
        try!(buffer.write_u16::<LittleEndian>(v));
        for &byte in buffer.iter() {
            self.output.push(byte);
        }
        Ok(())
    }

    fn emit_u8(&mut self, v: u8) -> Result<(), Self::Error> {
        try!(self.write_size(1));
        let mut buffer = vec![];
        try!(buffer.write_u8(v));
        for &byte in buffer.iter() {
            self.output.push(byte);
        }
        Ok(())
    }

    fn emit_isize(&mut self, _: isize) -> Result<(), Self::Error> {
        Err(Error::UnsupportedData)
    }

    fn emit_i64(&mut self, v: i64) -> Result<(), Self::Error> {
        try!(self.write_size(8));
        let mut buffer = vec![];
        try!(buffer.write_i64::<LittleEndian>(v));
        for &byte in buffer.iter() {
            self.output.push(byte);
        }
        Ok(())
    }

    fn emit_i32(&mut self, v: i32) -> Result<(), Self::Error> {
        try!(self.write_size(4));
        let mut buffer = vec![];
        try!(buffer.write_i32::<LittleEndian>(v));
        for &byte in buffer.iter() {
            self.output.push(byte);
        }
        Ok(())
    }

    fn emit_i16(&mut self, v: i16) -> Result<(), Self::Error> {
        try!(self.write_size(2));
        let mut buffer = vec![];
        try!(buffer.write_i16::<LittleEndian>(v));
        for &byte in buffer.iter() {
            self.output.push(byte);
        }
        Ok(())
    }

    fn emit_i8(&mut self, v: i8) -> Result<(), Self::Error> {
        try!(self.write_size(1));
        let mut buffer = vec![];
        try!(buffer.write_i8(v));
        for &byte in buffer.iter() {
            self.output.push(byte);
        }
        Ok(())
    }

    fn emit_bool(&mut self, v: bool) -> Result<(), Self::Error> {
        try!(self.write_size(1));
        self.output.push(if v {
            1u8
        } else {
            0u8
        });
        Ok(())
    }

    fn emit_f64(&mut self, v: f64) -> Result<(), Self::Error> {
        try!(self.write_size(8));
        let mut buffer = vec![];
        try!(buffer.write_f64::<LittleEndian>(v));
        for &byte in buffer.iter() {
            self.output.push(byte);
        }
        Ok(())
    }

    fn emit_f32(&mut self, v: f32) -> Result<(), Self::Error> {
        try!(self.write_size(4));
        let mut buffer = vec![];
        try!(buffer.write_f32::<LittleEndian>(v));
        for &byte in buffer.iter() {
            self.output.push(byte);
        }
        Ok(())
    }

    fn emit_char(&mut self, _: char) -> Result<(), Self::Error> {
        Err(Error::UnsupportedData)
    }

    fn emit_str(&mut self, v: &str) -> Result<(), Self::Error> {
        try!(self.write_size(v.len()));
        for &byte in v.as_bytes() {
            self.output.push(byte);
        }
        Ok(())
    }

    fn emit_enum<F>(&mut self, _: &str, _: F) -> Result<(), Self::Error>
        where F: FnOnce(&mut Self) -> Result<(), Self::Error>
    {
        Err(Error::UnsupportedData)
    }

    fn emit_enum_variant<F>(&mut self, _: &str, _: usize, _: usize, _: F) -> Result<(), Self::Error>
        where F: FnOnce(&mut Self) -> Result<(), Self::Error>
    {
        Err(Error::UnsupportedData)
    }

    fn emit_enum_variant_arg<F>(&mut self, _: usize, _: F) -> Result<(), Self::Error>
        where F: FnOnce(&mut Self) -> Result<(), Self::Error>
    {
        Err(Error::UnsupportedData)
    }

    fn emit_enum_struct_variant<F>(&mut self,
                                   _: &str,
                                   _: usize,
                                   _: usize,
                                   _: F)
                                   -> Result<(), Self::Error>
        where F: FnOnce(&mut Self) -> Result<(), Self::Error>
    {
        Err(Error::UnsupportedData)
    }

    fn emit_enum_struct_variant_field<F>(&mut self,
                                         _: &str,
                                         _: usize,
                                         _: F)
                                         -> Result<(), Self::Error>
        where F: FnOnce(&mut Self) -> Result<(), Self::Error>
    {
        Err(Error::UnsupportedData)
    }

    fn emit_struct<F>(&mut self, _: &str, _: usize, f: F) -> Result<(), Self::Error>
        where F: FnOnce(&mut Self) -> Result<(), Self::Error>
    {
        let start_point = self.output.len();
        for _ in 0..4 {
            self.output.push(0);
        }
        let inner_length = self.output.len();
        try!(f(self));
        let inner_length = self.output.len() - inner_length;
        try!(self.write_size_in_middle(start_point, inner_length));
        Ok(())
    }

    fn emit_struct_field<F>(&mut self, _: &str, _: usize, f: F) -> Result<(), Self::Error>
        where F: FnOnce(&mut Self) -> Result<(), Self::Error>
    {
        f(self)
    }

    fn emit_tuple<F>(&mut self, _: usize, f: F) -> Result<(), Self::Error>
        where F: FnOnce(&mut Self) -> Result<(), Self::Error>
    {
        let start_point = self.output.len();
        for _ in 0..4 {
            self.output.push(0);
        }
        let inner_length = self.output.len();
        try!(f(self));
        let inner_length = self.output.len() - inner_length;
        try!(self.write_size_in_middle(start_point, inner_length));
        Ok(())
    }

    fn emit_tuple_arg<F>(&mut self, _: usize, f: F) -> Result<(), Self::Error>
        where F: FnOnce(&mut Self) -> Result<(), Self::Error>
    {
        f(self)
    }

    fn emit_tuple_struct<F>(&mut self, _: &str, _: usize, _: F) -> Result<(), Self::Error>
        where F: FnOnce(&mut Self) -> Result<(), Self::Error>
    {
        Err(Error::UnsupportedData)
    }

    fn emit_tuple_struct_arg<F>(&mut self, _: usize, _: F) -> Result<(), Self::Error>
        where F: FnOnce(&mut Self) -> Result<(), Self::Error>
    {
        Err(Error::UnsupportedData)
    }

    fn emit_option<F>(&mut self, _: F) -> Result<(), Self::Error>
        where F: FnOnce(&mut Self) -> Result<(), Self::Error>
    {
        Err(Error::UnsupportedData)
    }

    fn emit_option_none(&mut self) -> Result<(), Self::Error> {
        Err(Error::UnsupportedData)
    }

    fn emit_option_some<F>(&mut self, _: F) -> Result<(), Self::Error>
        where F: FnOnce(&mut Self) -> Result<(), Self::Error>
    {
        Err(Error::UnsupportedData)
    }

    fn emit_seq<F>(&mut self, len: usize, f: F) -> Result<(), Self::Error>
        where F: FnOnce(&mut Self) -> Result<(), Self::Error>
    {
        let start_point = self.output.len();
        for _ in 0..4 {
            self.output.push(0);
        }
        let inner_length = self.output.len();
        try!(self.write_size(len));
        try!(f(self));
        let inner_length = self.output.len() - inner_length;
        try!(self.write_size_in_middle(start_point, inner_length));
        Ok(())
    }

    fn emit_seq_elt<F>(&mut self, _: usize, f: F) -> Result<(), Self::Error>
        where F: FnOnce(&mut Self) -> Result<(), Self::Error>
    {
        f(self)
    }

    fn emit_map<F>(&mut self, _: usize, _: F) -> Result<(), Self::Error>
        where F: FnOnce(&mut Self) -> Result<(), Self::Error>
    {
        Err(Error::UnsupportedData)
    }

    fn emit_map_elt_key<F>(&mut self, _: usize, _: F) -> Result<(), Self::Error>
        where F: FnOnce(&mut Self) -> Result<(), Self::Error>
    {
        Err(Error::UnsupportedData)
    }

    fn emit_map_elt_val<F>(&mut self, _: usize, _: F) -> Result<(), Self::Error>
        where F: FnOnce(&mut Self) -> Result<(), Self::Error>
    {
        Err(Error::UnsupportedData)
    }
}
