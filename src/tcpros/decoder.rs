use byteorder::{LittleEndian, ReadBytesExt};
use rustc_serialize;
use std;
use std::io::Read;
use self::error::{Error, ErrorKind, ResultExt};

pub struct DecoderSource<T>
    where T: std::io::Read
{
    input: T,
}

impl<T> DecoderSource<T>
    where T: std::io::Read
{
    pub fn new(data: T) -> DecoderSource<T> {
        DecoderSource { input: data }
    }

    pub fn pop_verification_byte(&mut self) -> Result<bool, std::io::Error> {
        self.input.read_u8().map(|v| v != 0)
    }

    pub fn pop_length(&mut self) -> Result<u32, std::io::Error> {
        self.input.read_u32::<LittleEndian>()
    }

    pub fn pop_decoder(&mut self) -> Result<Decoder, std::io::Error> {
        self.pop_length()
            .and_then(|length| {
                let mut data = vec![0u8; length as usize];
                self.input.read_exact(&mut data)?;
                Ok(Decoder {
                    input: std::io::Cursor::new(data),
                    extra: Some(length),
                })
            })
    }
}

impl<T> Iterator for DecoderSource<T>
    where T: std::io::Read
{
    type Item = Decoder;

    fn next(&mut self) -> Option<Decoder> {
        self.pop_decoder().ok()
    }
}

pub struct Decoder {
    input: std::io::Cursor<Vec<u8>>,
    extra: Option<u32>,
}

impl Decoder {
    pub fn pop_length(&mut self) -> Result<u32, std::io::Error> {
        match self.extra {
            Some(l) => {
                self.extra = None;
                Ok(l)
            }
            None => self.input.read_u32::<LittleEndian>(),
        }
    }
}

impl rustc_serialize::Decoder for Decoder {
    type Error = Error;

    fn read_nil(&mut self) -> Result<(), Self::Error> {
        bail!(ErrorKind::UnsupportedDataType("nil".into()))
    }

    fn read_usize(&mut self) -> Result<usize, Self::Error> {
        bail!(ErrorKind::UnsupportedDataType("usize".into()))
    }

    fn read_u64(&mut self) -> Result<u64, Self::Error> {
        self.input.read_u64::<LittleEndian>().chain_err(|| ErrorKind::EndOfBuffer)
    }

    fn read_u32(&mut self) -> Result<u32, Self::Error> {
        self.input.read_u32::<LittleEndian>().chain_err(|| ErrorKind::EndOfBuffer)
    }

    fn read_u16(&mut self) -> Result<u16, Self::Error> {
        self.input.read_u16::<LittleEndian>().chain_err(|| ErrorKind::EndOfBuffer)
    }

    fn read_u8(&mut self) -> Result<u8, Self::Error> {
        self.input.read_u8().chain_err(|| ErrorKind::EndOfBuffer)
    }

    fn read_isize(&mut self) -> Result<isize, Self::Error> {
        bail!(ErrorKind::UnsupportedDataType("isize".into()))
    }

    fn read_i64(&mut self) -> Result<i64, Self::Error> {
        self.input.read_i64::<LittleEndian>().chain_err(|| ErrorKind::EndOfBuffer)
    }

    fn read_i32(&mut self) -> Result<i32, Self::Error> {
        self.input.read_i32::<LittleEndian>().chain_err(|| ErrorKind::EndOfBuffer)
    }

    fn read_i16(&mut self) -> Result<i16, Self::Error> {
        self.input.read_i16::<LittleEndian>().chain_err(|| ErrorKind::EndOfBuffer)
    }

    fn read_i8(&mut self) -> Result<i8, Self::Error> {
        self.input.read_i8().chain_err(|| ErrorKind::EndOfBuffer)
    }

    fn read_bool(&mut self) -> Result<bool, Self::Error> {
        self.input.read_u8().chain_err(|| ErrorKind::EndOfBuffer).map(|v| v != 0)
    }

    fn read_f64(&mut self) -> Result<f64, Self::Error> {
        self.input.read_f64::<LittleEndian>().chain_err(|| ErrorKind::EndOfBuffer)
    }

    fn read_f32(&mut self) -> Result<f32, Self::Error> {
        self.input.read_f32::<LittleEndian>().chain_err(|| ErrorKind::EndOfBuffer)
    }

    fn read_char(&mut self) -> Result<char, Self::Error> {
        bail!(ErrorKind::UnsupportedDataType("char".into()))
    }

    fn read_str(&mut self) -> Result<String, Self::Error> {
        let length = self.pop_length().chain_err(|| ErrorKind::EndOfBuffer)?;
        let mut buffer = vec![0; length as usize];
        self.input.read_exact(&mut buffer).chain_err(|| ErrorKind::EndOfBuffer)?;
        String::from_utf8(buffer)
            .chain_err(|| ErrorKind::UnsupportedDataType("non-UTF-8 string".into()))
    }

    fn read_enum<T, F>(&mut self, _: &str, _: F) -> Result<T, Self::Error>
        where F: FnOnce(&mut Self) -> Result<T, Self::Error>
    {
        bail!(ErrorKind::UnsupportedDataType("enum".into()))
    }

    fn read_enum_variant<T, F>(&mut self, _: &[&str], _: F) -> Result<T, Self::Error>
        where F: FnMut(&mut Self, usize) -> Result<T, Self::Error>
    {
        bail!(ErrorKind::UnsupportedDataType("enum variant".into()))
    }

    fn read_enum_variant_arg<T, F>(&mut self, _: usize, _: F) -> Result<T, Self::Error>
        where F: FnOnce(&mut Self) -> Result<T, Self::Error>
    {
        bail!(ErrorKind::UnsupportedDataType("enum variant argument".into()))
    }

    fn read_enum_struct_variant<T, F>(&mut self, _: &[&str], _: F) -> Result<T, Self::Error>
        where F: FnMut(&mut Self, usize) -> Result<T, Self::Error>
    {
        bail!(ErrorKind::UnsupportedDataType("enum struct variant".into()))
    }

    fn read_enum_struct_variant_field<T, F>(&mut self,
                                            _: &str,
                                            _: usize,
                                            _: F)
                                            -> Result<T, Self::Error>
        where F: FnOnce(&mut Self) -> Result<T, Self::Error>
    {
        bail!(ErrorKind::UnsupportedDataType("enum struct variant field".into()))
    }

    fn read_struct<T, F>(&mut self, name: &str, _: usize, f: F) -> Result<T, Self::Error>
        where F: FnOnce(&mut Self) -> Result<T, Self::Error>
    {
        self.pop_length().chain_err(|| ErrorKind::EndOfBuffer)?;
        f(self).chain_err(|| ErrorKind::FailedToDecode(format!("struct {}", name)))
    }

    fn read_struct_field<T, F>(&mut self, name: &str, _: usize, f: F) -> Result<T, Self::Error>
        where F: FnOnce(&mut Self) -> Result<T, Self::Error>
    {
        f(self).chain_err(|| ErrorKind::FailedToDecode(format!("field {}", name)))
    }

    fn read_tuple<T, F>(&mut self, _: usize, f: F) -> Result<T, Self::Error>
        where F: FnOnce(&mut Self) -> Result<T, Self::Error>
    {
        self.pop_length().chain_err(|| ErrorKind::EndOfBuffer)?;
        f(self).chain_err(|| ErrorKind::FailedToDecode("tuple".into()))
    }

    fn read_tuple_arg<T, F>(&mut self, n: usize, f: F) -> Result<T, Self::Error>
        where F: FnOnce(&mut Self) -> Result<T, Self::Error>
    {
        f(self).chain_err(|| ErrorKind::FailedToDecode(format!("field number {}", n)))
    }

    fn read_tuple_struct<T, F>(&mut self, _: &str, _: usize, _: F) -> Result<T, Self::Error>
        where F: FnOnce(&mut Self) -> Result<T, Self::Error>
    {
        bail!(ErrorKind::UnsupportedDataType("tuple struct".into()))
    }

    fn read_tuple_struct_arg<T, F>(&mut self, _: usize, _: F) -> Result<T, Self::Error>
        where F: FnOnce(&mut Self) -> Result<T, Self::Error>
    {
        bail!(ErrorKind::UnsupportedDataType("tuple struct argument".into()))
    }

    fn read_option<T, F>(&mut self, _: F) -> Result<T, Self::Error>
        where F: FnMut(&mut Self, bool) -> Result<T, Self::Error>
    {
        bail!(ErrorKind::UnsupportedDataType("option".into()))
    }

    fn read_seq<T, F>(&mut self, f: F) -> Result<T, Self::Error>
        where F: FnOnce(&mut Self, usize) -> Result<T, Self::Error>
    {
        self.pop_length().chain_err(|| ErrorKind::EndOfBuffer)?;
        let count = self.pop_length().chain_err(|| ErrorKind::EndOfBuffer)? as usize;
        f(self, count).chain_err(|| ErrorKind::FailedToDecode("array".into()))
    }

    fn read_seq_elt<T, F>(&mut self, _: usize, f: F) -> Result<T, Self::Error>
        where F: FnOnce(&mut Self) -> Result<T, Self::Error>
    {
        f(self).chain_err(|| ErrorKind::FailedToDecode("array element".into()))
    }

    fn read_map<T, F>(&mut self, _: F) -> Result<T, Self::Error>
        where F: FnOnce(&mut Self, usize) -> Result<T, Self::Error>
    {
        bail!(ErrorKind::UnsupportedDataType("map".into()))
    }

    fn read_map_elt_key<T, F>(&mut self, _: usize, _: F) -> Result<T, Self::Error>
        where F: FnOnce(&mut Self) -> Result<T, Self::Error>
    {
        bail!(ErrorKind::UnsupportedDataType("map element key".into()))
    }

    fn read_map_elt_val<T, F>(&mut self, _: usize, _: F) -> Result<T, Self::Error>
        where F: FnOnce(&mut Self) -> Result<T, Self::Error>
    {
        bail!(ErrorKind::UnsupportedDataType("map element value".into()))
    }

    fn error(&mut self, err: &str) -> Self::Error {
        err.into()
    }
}

pub mod error {
    error_chain! {
        errors {
            UnsupportedDataType(t: String) {
                description("Datatype is not supported")
                display("Datatype is not supported, issue within {}", t)
            }
            FailedToDecode(t: String) {
                description("Failed to decode")
                display("Failed to decode {}", t)
            }
            EndOfBuffer {
                description("Reached end of memory buffer")
                display("Reached end of memory buffer while reading data")
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std;
    use rustc_serialize::Decodable;

    fn push_data(data: Vec<u8>) -> DecoderSource<std::io::Cursor<Vec<u8>>> {
        DecoderSource::new(std::io::Cursor::new(data))
    }

    #[test]
    fn pops_length_right() {
        let mut decoder = push_data(vec![4, 0, 0, 0, 2, 33, 17, 0]);
        assert_eq!(4, decoder.pop_length().unwrap());
        assert_eq!(1122562, decoder.pop_length().unwrap());
    }

    #[test]
    fn reads_u8() {
        let mut decoder = push_data(vec![1, 0, 0, 0, 150]);
        assert_eq!(150, u8::decode(&mut decoder.next().unwrap()).unwrap());
    }

    #[test]
    fn reads_u16() {
        let mut decoder = push_data(vec![2, 0, 0, 0, 0x34, 0xA2]);
        assert_eq!(0xA234, u16::decode(&mut decoder.next().unwrap()).unwrap());
    }

    #[test]
    fn reads_u32() {
        let mut decoder = push_data(vec![4, 0, 0, 0, 0x45, 0x23, 1, 0xCD]);
        assert_eq!(0xCD012345,
                   u32::decode(&mut decoder.next().unwrap()).unwrap());
    }

    #[test]
    fn reads_u64() {
        let mut decoder = push_data(vec![8, 0, 0, 0, 0xBB, 0xAA, 0x10, 0x32, 0x54, 0x76, 0x98,
                                         0xAB]);
        assert_eq!(0xAB9876543210AABB,
                   u64::decode(&mut decoder.next().unwrap()).unwrap());
    }

    #[test]
    fn reads_i8() {
        let mut decoder = push_data(vec![1, 0, 0, 0, 156]);
        assert_eq!(-100, i8::decode(&mut decoder.next().unwrap()).unwrap());
    }

    #[test]
    fn reads_i16() {
        let mut decoder = push_data(vec![2, 0, 0, 0, 0xD0, 0x8A]);
        assert_eq!(-30000, i16::decode(&mut decoder.next().unwrap()).unwrap());
    }

    #[test]
    fn reads_i32() {
        let mut decoder = push_data(vec![4, 0, 0, 0, 0x00, 0x6C, 0xCA, 0x88]);
        assert_eq!(-2000000000,
                   i32::decode(&mut decoder.next().unwrap()).unwrap());
    }

    #[test]
    fn reads_i64() {
        let mut decoder = push_data(vec![8, 0, 0, 0, 0x00, 0x00, 0x7c, 0x1d, 0xaf, 0x93, 0x19,
                                         0x83]);
        assert_eq!(-9000000000000000000,
                   i64::decode(&mut decoder.next().unwrap()).unwrap());
    }

    #[test]
    fn reads_f32() {
        let mut decoder = push_data(vec![4, 0, 0, 0, 0x00, 0x70, 0x7b, 0x44]);
        assert_eq!(1005.75, f32::decode(&mut decoder.next().unwrap()).unwrap());
    }

    #[test]
    fn reads_f64() {
        let mut decoder = push_data(vec![8, 0, 0, 0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x6e, 0x8f,
                                         0x40]);
        assert_eq!(1005.75, f64::decode(&mut decoder.next().unwrap()).unwrap());
    }

    #[test]
    fn reads_bool() {
        let mut decoder = push_data(vec![1, 0, 0, 0, 1]);
        assert_eq!(true, bool::decode(&mut decoder.next().unwrap()).unwrap());
        let mut decoder = push_data(vec![1, 0, 0, 0, 0]);
        assert_eq!(false, bool::decode(&mut decoder.next().unwrap()).unwrap());
    }

    #[test]
    fn reads_string() {
        let mut decoder = push_data(vec![0, 0, 0, 0]);
        assert_eq!("", String::decode(&mut decoder.next().unwrap()).unwrap());
        let mut decoder = push_data(vec![13, 0, 0, 0, 72, 101, 108, 108, 111, 44, 32, 87, 111,
                                         114, 108, 100, 33]);
        assert_eq!("Hello, World!",
                   String::decode(&mut decoder.next().unwrap()).unwrap());
    }

    #[test]
    fn reads_array() {
        let mut decoder = push_data(vec![12, 0, 0, 0, 4, 0, 0, 0, 7, 0, 1, 4, 33, 0, 57, 0]);
        assert_eq!(vec![7, 1025, 33, 57],
                   Vec::<i16>::decode(&mut decoder.next().unwrap()).unwrap());
    }

    #[derive(Debug,RustcDecodable,PartialEq)]
    struct TestStructOne {
        a: i16,
        b: bool,
        c: u8,
        d: String,
        e: Vec<bool>,
    }

    #[test]
    fn reads_simple_struct() {
        let v = TestStructOne {
            a: 2050i16,
            b: true,
            c: 7u8,
            d: String::from("ABC012"),
            e: vec![true, false, false, true],
        };
        let mut decoder = push_data(vec![26, 0, 0, 0, 2, 8, 1, 7, 6, 0, 0, 0, 65, 66, 67, 48, 49,
                                         50, 8, 0, 0, 0, 4, 0, 0, 0, 1, 0, 0, 1]);
        assert_eq!(v,
                   TestStructOne::decode(&mut decoder.next().unwrap()).unwrap());
    }

    #[derive(Debug,RustcDecodable,PartialEq)]
    struct TestStructPart {
        a: String,
        b: bool,
    }

    #[derive(Debug,RustcDecodable,PartialEq)]
    struct TestStructBig {
        a: Vec<TestStructPart>,
        b: String,
    }

    #[test]
    fn reads_complex_struct() {
        let mut parts = Vec::new();
        parts.push(TestStructPart {
            a: String::from("ABC"),
            b: true,
        });
        parts.push(TestStructPart {
            a: String::from("1!!!!"),
            b: true,
        });
        parts.push(TestStructPart {
            a: String::from("234b"),
            b: false,
        });
        let v = TestStructBig {
            a: parts,
            b: String::from("EEe"),
        };
        let mut decoder = push_data(vec![54, 0, 0, 0, 43, 0, 0, 0, 3, 0, 0, 0, 8, 0, 0, 0, 3, 0,
                                         0, 0, 65, 66, 67, 1, 10, 0, 0, 0, 5, 0, 0, 0, 49, 33,
                                         33, 33, 33, 1, 9, 0, 0, 0, 4, 0, 0, 0, 50, 51, 52, 98,
                                         0, 3, 0, 0, 0, 69, 69, 101]);
        assert_eq!(v,
                   TestStructBig::decode(&mut decoder.next().unwrap()).unwrap());
    }
}
