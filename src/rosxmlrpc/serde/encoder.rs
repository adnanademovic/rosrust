extern crate xml;
extern crate rustc_serialize;

use std;

pub struct Encoder<T: std::io::Write> {
    writer: xml::EventWriter<T>,
}

impl<T: std::io::Write> Encoder<T> {
    pub fn new(body: T) -> Encoder<T> {
        Encoder::<T> { writer: xml::EventWriter::new(body) }
    }

    pub fn start_request(&mut self, function_name: &str) -> Result<(), Error> {
        try!(self.writer.write(xml::writer::XmlEvent::start_element("methodCall")));
        try!(self.writer.write(xml::writer::XmlEvent::start_element("methodName")));
        try!(self.writer.write(xml::writer::XmlEvent::characters(function_name)));
        try!(self.writer.write(xml::writer::XmlEvent::end_element()));
        try!(self.writer.write(xml::writer::XmlEvent::start_element("params")));
        Ok(())
    }

    pub fn end_request(&mut self) -> Result<(), Error> {
        try!(self.writer.write(xml::writer::XmlEvent::end_element()));
        try!(self.writer.write(xml::writer::XmlEvent::end_element()));
        Ok(())
    }

    pub fn start_response(&mut self) -> Result<(), Error> {
        try!(self.writer.write(xml::writer::XmlEvent::start_element("methodResponse")));
        try!(self.writer.write(xml::writer::XmlEvent::start_element("params")));
        try!(self.writer.write(xml::writer::XmlEvent::start_element("param")));
        Ok(())
    }

    pub fn end_response(&mut self) -> Result<(), Error> {
        try!(self.writer.write(xml::writer::XmlEvent::end_element()));
        try!(self.writer.write(xml::writer::XmlEvent::end_element()));
        try!(self.writer.write(xml::writer::XmlEvent::end_element()));
        Ok(())
    }

    fn encoder_emit_array<F>(&mut self, f: F) -> Result<(), Error>
        where F: FnOnce(&mut Self) -> Result<(), Error>
    {
        try!(self.writer.write(xml::writer::XmlEvent::start_element("value")));
        try!(self.writer.write(xml::writer::XmlEvent::start_element("array")));
        try!(self.writer.write(xml::writer::XmlEvent::start_element("data")));
        try!(f(self));
        try!(self.writer.write(xml::writer::XmlEvent::end_element()));
        try!(self.writer.write(xml::writer::XmlEvent::end_element()));
        try!(self.writer.write(xml::writer::XmlEvent::end_element()));
        Ok(())
    }
}

impl<T: std::io::Write> rustc_serialize::Encoder for Encoder<T> {
    type Error = Error;

    fn emit_nil(&mut self) -> Result<(), Self::Error> {
        Err(Error::UnsupportedDataFormat)
    }

    fn emit_usize(&mut self, _: usize) -> Result<(), Self::Error> {
        Err(Error::UnsupportedDataFormat)
    }

    fn emit_u64(&mut self, _: u64) -> Result<(), Self::Error> {
        Err(Error::UnsupportedDataFormat)
    }

    fn emit_u32(&mut self, _: u32) -> Result<(), Self::Error> {
        Err(Error::UnsupportedDataFormat)
    }

    fn emit_u16(&mut self, _: u16) -> Result<(), Self::Error> {
        Err(Error::UnsupportedDataFormat)
    }

    fn emit_u8(&mut self, _: u8) -> Result<(), Self::Error> {
        Err(Error::UnsupportedDataFormat)
    }

    fn emit_isize(&mut self, _: isize) -> Result<(), Self::Error> {
        Err(Error::UnsupportedDataFormat)
    }

    fn emit_i64(&mut self, _: i64) -> Result<(), Self::Error> {
        Err(Error::UnsupportedDataFormat)
    }

    fn emit_i32(&mut self, v: i32) -> Result<(), Self::Error> {
        try!(self.writer.write(xml::writer::XmlEvent::start_element("value")));
        try!(self.writer.write(xml::writer::XmlEvent::start_element("int")));
        try!(self.writer.write(xml::writer::XmlEvent::characters(&v.to_string())));
        try!(self.writer.write(xml::writer::XmlEvent::end_element()));
        try!(self.writer.write(xml::writer::XmlEvent::end_element()));
        Ok(())
    }

    fn emit_i16(&mut self, _: i16) -> Result<(), Self::Error> {
        Err(Error::UnsupportedDataFormat)
    }

    fn emit_i8(&mut self, _: i8) -> Result<(), Self::Error> {
        Err(Error::UnsupportedDataFormat)
    }

    fn emit_bool(&mut self, v: bool) -> Result<(), Self::Error> {
        try!(self.writer.write(xml::writer::XmlEvent::start_element("value")));
        try!(self.writer.write(xml::writer::XmlEvent::start_element("boolean")));
        try!(self.writer.write(xml::writer::XmlEvent::characters({
            if v {
                "1"
            } else {
                "0"
            }
        })));
        try!(self.writer.write(xml::writer::XmlEvent::end_element()));
        try!(self.writer.write(xml::writer::XmlEvent::end_element()));
        Ok(())
    }

    fn emit_f64(&mut self, v: f64) -> Result<(), Self::Error> {
        try!(self.writer.write(xml::writer::XmlEvent::start_element("value")));
        try!(self.writer.write(xml::writer::XmlEvent::start_element("double")));
        try!(self.writer.write(xml::writer::XmlEvent::characters(&v.to_string())));
        try!(self.writer.write(xml::writer::XmlEvent::end_element()));
        try!(self.writer.write(xml::writer::XmlEvent::end_element()));
        Ok(())
    }

    fn emit_f32(&mut self, _: f32) -> Result<(), Self::Error> {
        Err(Error::UnsupportedDataFormat)
    }

    fn emit_char(&mut self, _: char) -> Result<(), Self::Error> {
        Err(Error::UnsupportedDataFormat)
    }

    fn emit_str(&mut self, v: &str) -> Result<(), Self::Error> {
        try!(self.writer.write(xml::writer::XmlEvent::start_element("value")));
        try!(self.writer.write(xml::writer::XmlEvent::start_element("string")));
        try!(self.writer.write(xml::writer::XmlEvent::characters(v)));
        try!(self.writer.write(xml::writer::XmlEvent::end_element()));
        try!(self.writer.write(xml::writer::XmlEvent::end_element()));
        Ok(())
    }

    fn emit_enum<F>(&mut self, _: &str, _: F) -> Result<(), Self::Error>
        where F: FnOnce(&mut Self) -> Result<(), Self::Error>
    {
        Err(Error::UnsupportedDataFormat)
    }

    fn emit_enum_variant<F>(&mut self, _: &str, _: usize, _: usize, _: F) -> Result<(), Self::Error>
        where F: FnOnce(&mut Self) -> Result<(), Self::Error>
    {
        Err(Error::UnsupportedDataFormat)
    }

    fn emit_enum_variant_arg<F>(&mut self, _: usize, _: F) -> Result<(), Self::Error>
        where F: FnOnce(&mut Self) -> Result<(), Self::Error>
    {
        Err(Error::UnsupportedDataFormat)
    }

    fn emit_enum_struct_variant<F>(&mut self,
                                   _: &str,
                                   _: usize,
                                   _: usize,
                                   _: F)
                                   -> Result<(), Self::Error>
        where F: FnOnce(&mut Self) -> Result<(), Self::Error>
    {
        Err(Error::UnsupportedDataFormat)
    }

    fn emit_enum_struct_variant_field<F>(&mut self,
                                         _: &str,
                                         _: usize,
                                         _: F)
                                         -> Result<(), Self::Error>
        where F: FnOnce(&mut Self) -> Result<(), Self::Error>
    {
        Err(Error::UnsupportedDataFormat)
    }

    fn emit_struct<F>(&mut self, _: &str, _: usize, f: F) -> Result<(), Self::Error>
        where F: FnOnce(&mut Self) -> Result<(), Self::Error>
    {
        self.encoder_emit_array(f)
    }

    fn emit_struct_field<F>(&mut self, _: &str, _: usize, f: F) -> Result<(), Self::Error>
        where F: FnOnce(&mut Self) -> Result<(), Self::Error>
    {
        f(self)
    }

    fn emit_tuple<F>(&mut self, _: usize, f: F) -> Result<(), Self::Error>
        where F: FnOnce(&mut Self) -> Result<(), Self::Error>
    {
        self.encoder_emit_array(f)
    }

    fn emit_tuple_arg<F>(&mut self, _: usize, f: F) -> Result<(), Self::Error>
        where F: FnOnce(&mut Self) -> Result<(), Self::Error>
    {
        f(self)
    }

    fn emit_tuple_struct<F>(&mut self, _: &str, _: usize, f: F) -> Result<(), Self::Error>
        where F: FnOnce(&mut Self) -> Result<(), Self::Error>
    {
        self.encoder_emit_array(f)
    }

    fn emit_tuple_struct_arg<F>(&mut self, _: usize, f: F) -> Result<(), Self::Error>
        where F: FnOnce(&mut Self) -> Result<(), Self::Error>
    {
        f(self)
    }

    fn emit_option<F>(&mut self, _: F) -> Result<(), Self::Error>
        where F: FnOnce(&mut Self) -> Result<(), Self::Error>
    {
        Err(Error::UnsupportedDataFormat)
    }

    fn emit_option_none(&mut self) -> Result<(), Self::Error> {
        Err(Error::UnsupportedDataFormat)
    }

    fn emit_option_some<F>(&mut self, _: F) -> Result<(), Self::Error>
        where F: FnOnce(&mut Self) -> Result<(), Self::Error>
    {
        Err(Error::UnsupportedDataFormat)
    }

    fn emit_seq<F>(&mut self, _: usize, f: F) -> Result<(), Self::Error>
        where F: FnOnce(&mut Self) -> Result<(), Self::Error>
    {
        self.encoder_emit_array(f)
    }

    fn emit_seq_elt<F>(&mut self, _: usize, f: F) -> Result<(), Self::Error>
        where F: FnOnce(&mut Self) -> Result<(), Self::Error>
    {
        f(self)
    }

    fn emit_map<F>(&mut self, _: usize, _: F) -> Result<(), Self::Error>
        where F: FnOnce(&mut Self) -> Result<(), Self::Error>
    {
        Err(Error::UnsupportedDataFormat)
    }

    fn emit_map_elt_key<F>(&mut self, _: usize, _: F) -> Result<(), Self::Error>
        where F: FnOnce(&mut Self) -> Result<(), Self::Error>
    {
        Err(Error::UnsupportedDataFormat)
    }

    fn emit_map_elt_val<F>(&mut self, _: usize, _: F) -> Result<(), Self::Error>
        where F: FnOnce(&mut Self) -> Result<(), Self::Error>
    {
        Err(Error::UnsupportedDataFormat)
    }
}

#[derive(Debug)]
pub enum Error {
    XmlWrite(xml::writer::Error),
    UnsupportedDataFormat,
}

impl From<xml::writer::Error> for Error {
    fn from(err: xml::writer::Error) -> Error {
        Error::XmlWrite(err)
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            Error::XmlWrite(ref err) => write!(f, "XML writing error: {}", err),
            Error::UnsupportedDataFormat => write!(f, "Unencodable data provided"),
        }
    }
}

impl std::error::Error for Error {
    fn description(&self) -> &str {
        match *self {
            Error::XmlWrite(..) => "Descriptions not implemented for xml::writer::Error",
            Error::UnsupportedDataFormat => "Unencodable data provided",
        }
    }

    fn cause(&self) -> Option<&std::error::Error> {
        match *self {
            Error::XmlWrite(..) => None,
            Error::UnsupportedDataFormat => None,
        }
    }
}
