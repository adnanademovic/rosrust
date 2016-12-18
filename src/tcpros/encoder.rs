use byteorder::{LittleEndian, WriteBytesExt};
use rustc_serialize;
use std;
use super::error::Error;

#[derive(Debug)]
pub struct Encoder {
    output: Vec<Vec<u8>>,
}

impl Encoder {
    pub fn new() -> Encoder {
        Encoder { output: Vec::<Vec<u8>>::new() }
    }

    pub fn len(&self) -> usize {
        self.output.iter().fold(0, |accum, ref v| accum + v.len())
    }

    pub fn write_to<T: std::io::Write>(self, output: &mut T) -> Result<(), std::io::Error> {
        for v in self.output {
            output.write_all(&v)?;
        }
        Ok(())
    }

    fn write_size(&mut self, v: usize) -> Result<(), Error> {
        let v = v as u32;
        let mut buffer = vec![];
        buffer.write_u32::<LittleEndian>(v)?;
        self.output.push(buffer);
        Ok(())
    }

    fn write_variable(&mut self, buffer: Vec<u8>) -> Result<(), Error> {
        self.write_size(buffer.len())?;
        self.output.push(buffer);
        Ok(())
    }

    fn write_size_in_middle(&mut self, position: usize, v: usize) -> Result<(), Error> {
        let v = v as u32;
        self.output
            .get_mut(position)
            .unwrap()
            .write_u32::<LittleEndian>(v)
            .map_err(|v| Error::Io(v))
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
        let mut buffer = vec![];
        buffer.write_u64::<LittleEndian>(v)?;
        self.write_variable(buffer)
    }

    fn emit_u32(&mut self, v: u32) -> Result<(), Self::Error> {
        let mut buffer = vec![];
        buffer.write_u32::<LittleEndian>(v)?;
        self.write_variable(buffer)
    }

    fn emit_u16(&mut self, v: u16) -> Result<(), Self::Error> {
        let mut buffer = vec![];
        buffer.write_u16::<LittleEndian>(v)?;
        self.write_variable(buffer)
    }

    fn emit_u8(&mut self, v: u8) -> Result<(), Self::Error> {
        self.write_variable(vec![v])
    }

    fn emit_isize(&mut self, _: isize) -> Result<(), Self::Error> {
        Err(Error::UnsupportedData)
    }

    fn emit_i64(&mut self, v: i64) -> Result<(), Self::Error> {
        let mut buffer = vec![];
        buffer.write_i64::<LittleEndian>(v)?;
        self.write_variable(buffer)
    }

    fn emit_i32(&mut self, v: i32) -> Result<(), Self::Error> {
        let mut buffer = vec![];
        buffer.write_i32::<LittleEndian>(v)?;
        self.write_variable(buffer)
    }

    fn emit_i16(&mut self, v: i16) -> Result<(), Self::Error> {
        let mut buffer = vec![];
        buffer.write_i16::<LittleEndian>(v)?;
        self.write_variable(buffer)
    }

    fn emit_i8(&mut self, v: i8) -> Result<(), Self::Error> {
        let mut buffer = vec![];
        buffer.write_i8(v)?;
        self.write_variable(buffer)
    }

    fn emit_bool(&mut self, v: bool) -> Result<(), Self::Error> {
        self.write_variable(vec![if v { 1u8 } else { 0u8 }])
    }

    fn emit_f64(&mut self, v: f64) -> Result<(), Self::Error> {
        let mut buffer = vec![];
        buffer.write_f64::<LittleEndian>(v)?;
        self.write_variable(buffer)
    }

    fn emit_f32(&mut self, v: f32) -> Result<(), Self::Error> {
        let mut buffer = vec![];
        buffer.write_f32::<LittleEndian>(v)?;
        self.write_variable(buffer)
    }

    fn emit_char(&mut self, _: char) -> Result<(), Self::Error> {
        Err(Error::UnsupportedData)
    }

    fn emit_str(&mut self, v: &str) -> Result<(), Self::Error> {
        self.write_variable(v.as_bytes().to_vec())
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

    fn emit_struct<F>(&mut self, _: &str, len: usize, f: F) -> Result<(), Self::Error>
        where F: FnOnce(&mut Self) -> Result<(), Self::Error>
    {
        self.emit_tuple(len, f)
    }

    fn emit_struct_field<F>(&mut self, _: &str, _: usize, f: F) -> Result<(), Self::Error>
        where F: FnOnce(&mut Self) -> Result<(), Self::Error>
    {
        f(self)
    }

    fn emit_tuple<F>(&mut self, _: usize, f: F) -> Result<(), Self::Error>
        where F: FnOnce(&mut Self) -> Result<(), Self::Error>
    {
        self.output.push(Vec::new());
        let position = self.output.len();
        f(self)?;
        let length = self.output[position..].iter().map(|v| v.len()).sum();
        self.write_size_in_middle(position - 1, length)
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
        self.output.push(Vec::new());
        let position = self.output.len();
        self.write_size(len)?;
        f(self)?;
        let length = self.output[position..].iter().map(|v| v.len()).sum();
        self.write_size_in_middle(position - 1, length)
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
