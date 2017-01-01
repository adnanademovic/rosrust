use byteorder::{LittleEndian, WriteBytesExt};
use rustc_serialize;
use std;
use self::error::{Error, ErrorKind, ResultExt};

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

    pub fn write_to<T: std::io::Write>(&self, output: &mut T) -> Result<(), std::io::Error> {
        for ref v in &self.output {
            output.write_all(&v)?;
        }
        Ok(())
    }

    fn emit_tuple_helper<F>(&mut self, f: F) -> EncoderResult
        where F: FnOnce(&mut Self) -> EncoderResult
    {
        self.output.push(Vec::new());
        let position = self.output.len();
        f(self)?;
        let length = self.output[position..].iter().map(|v| v.len()).sum();
        Ok(self.write_size_in_middle(position - 1, length))
    }

    fn write_size(&mut self, v: usize) {
        let v = v as u32;
        let mut buffer = vec![];
        // Failure here can only be caused by internal changes byteorder
        buffer.write_u32::<LittleEndian>(v).unwrap();
        self.output.push(buffer);
    }

    fn write_variable(&mut self, buffer: Vec<u8>) {
        self.output.push(buffer);
    }

    fn write_size_in_middle(&mut self, position: usize, v: usize) {
        let v = v as u32;
        self.output
            .get_mut(position)
            .unwrap()
            .write_u32::<LittleEndian>(v)
            .unwrap();
    }
}

type EncoderResult = Result<(), Error>;

impl rustc_serialize::Encoder for Encoder {
    type Error = Error;

    fn emit_nil(&mut self) -> EncoderResult {
        bail!(ErrorKind::UnsupportedDataType("nil".into()))
    }

    fn emit_usize(&mut self, _: usize) -> EncoderResult {
        bail!(ErrorKind::UnsupportedDataType("usize".into()))
    }

    fn emit_u64(&mut self, v: u64) -> EncoderResult {
        let mut buffer = vec![];
        buffer.write_u64::<LittleEndian>(v).unwrap();
        self.write_variable(buffer);
        Ok(())
    }

    fn emit_u32(&mut self, v: u32) -> EncoderResult {
        let mut buffer = vec![];
        buffer.write_u32::<LittleEndian>(v).unwrap();
        self.write_variable(buffer);
        Ok(())
    }

    fn emit_u16(&mut self, v: u16) -> EncoderResult {
        let mut buffer = vec![];
        buffer.write_u16::<LittleEndian>(v).unwrap();
        self.write_variable(buffer);
        Ok(())
    }

    fn emit_u8(&mut self, v: u8) -> EncoderResult {
        self.write_variable(vec![v]);
        Ok(())
    }

    fn emit_isize(&mut self, _: isize) -> EncoderResult {
        bail!(ErrorKind::UnsupportedDataType("isize".into()))
    }

    fn emit_i64(&mut self, v: i64) -> EncoderResult {
        let mut buffer = vec![];
        buffer.write_i64::<LittleEndian>(v).unwrap();
        self.write_variable(buffer);
        Ok(())
    }

    fn emit_i32(&mut self, v: i32) -> EncoderResult {
        let mut buffer = vec![];
        buffer.write_i32::<LittleEndian>(v).unwrap();
        self.write_variable(buffer);
        Ok(())
    }

    fn emit_i16(&mut self, v: i16) -> EncoderResult {
        let mut buffer = vec![];
        buffer.write_i16::<LittleEndian>(v).unwrap();
        self.write_variable(buffer);
        Ok(())
    }

    fn emit_i8(&mut self, v: i8) -> EncoderResult {
        let mut buffer = vec![];
        buffer.write_i8(v).unwrap();
        self.write_variable(buffer);
        Ok(())
    }

    fn emit_bool(&mut self, v: bool) -> EncoderResult {
        self.write_variable(vec![if v { 1u8 } else { 0u8 }]);
        Ok(())
    }

    fn emit_f64(&mut self, v: f64) -> EncoderResult {
        let mut buffer = vec![];
        buffer.write_f64::<LittleEndian>(v).unwrap();
        self.write_variable(buffer);
        Ok(())
    }

    fn emit_f32(&mut self, v: f32) -> EncoderResult {
        let mut buffer = vec![];
        buffer.write_f32::<LittleEndian>(v).unwrap();
        self.write_variable(buffer);
        Ok(())
    }

    fn emit_char(&mut self, _: char) -> EncoderResult {
        bail!(ErrorKind::UnsupportedDataType("char".into()))
    }

    fn emit_str(&mut self, v: &str) -> EncoderResult {
        let data = v.as_bytes().to_vec();
        self.write_size(data.len());
        self.write_variable(data);
        Ok(())
    }

    fn emit_enum<F>(&mut self, _: &str, _: F) -> EncoderResult
        where F: FnOnce(&mut Self) -> EncoderResult
    {
        bail!(ErrorKind::UnsupportedDataType("enum".into()))
    }

    fn emit_enum_variant<F>(&mut self, _: &str, _: usize, _: usize, _: F) -> EncoderResult
        where F: FnOnce(&mut Self) -> EncoderResult
    {
        bail!(ErrorKind::UnsupportedDataType("enum variant".into()))
    }

    fn emit_enum_variant_arg<F>(&mut self, _: usize, _: F) -> EncoderResult
        where F: FnOnce(&mut Self) -> EncoderResult
    {
        bail!(ErrorKind::UnsupportedDataType("enum variant argument".into()))
    }

    fn emit_enum_struct_variant<F>(&mut self, _: &str, _: usize, _: usize, _: F) -> EncoderResult
        where F: FnOnce(&mut Self) -> EncoderResult
    {
        bail!(ErrorKind::UnsupportedDataType("enum struct variant".into()))
    }

    fn emit_enum_struct_variant_field<F>(&mut self, _: &str, _: usize, _: F) -> EncoderResult
        where F: FnOnce(&mut Self) -> EncoderResult
    {
        bail!(ErrorKind::UnsupportedDataType("enum struct variant field".into()))
    }

    fn emit_struct<F>(&mut self, name: &str, _: usize, f: F) -> EncoderResult
        where F: FnOnce(&mut Self) -> EncoderResult
    {
        self.emit_tuple_helper(f)
            .chain_err(|| ErrorKind::UnsupportedDataType(format!("struct {}", name)))
    }

    fn emit_struct_field<F>(&mut self, name: &str, _: usize, f: F) -> EncoderResult
        where F: FnOnce(&mut Self) -> EncoderResult
    {
        f(self).chain_err(|| ErrorKind::UnsupportedDataType(format!("field {}", name)))
    }

    fn emit_tuple<F>(&mut self, _: usize, f: F) -> EncoderResult
        where F: FnOnce(&mut Self) -> EncoderResult
    {
        self.emit_tuple_helper(f).chain_err(|| ErrorKind::UnsupportedDataType("tuple".into()))
    }

    fn emit_tuple_arg<F>(&mut self, n: usize, f: F) -> EncoderResult
        where F: FnOnce(&mut Self) -> EncoderResult
    {
        f(self).chain_err(|| ErrorKind::UnsupportedDataType(format!("field number {}", n)))
    }

    fn emit_tuple_struct<F>(&mut self, _: &str, _: usize, _: F) -> EncoderResult
        where F: FnOnce(&mut Self) -> EncoderResult
    {
        bail!(ErrorKind::UnsupportedDataType("tuple structure".into()))
    }

    fn emit_tuple_struct_arg<F>(&mut self, _: usize, _: F) -> EncoderResult
        where F: FnOnce(&mut Self) -> EncoderResult
    {
        bail!(ErrorKind::UnsupportedDataType("tuple structure argument".into()))
    }

    fn emit_option<F>(&mut self, _: F) -> EncoderResult
        where F: FnOnce(&mut Self) -> EncoderResult
    {
        bail!(ErrorKind::UnsupportedDataType("option".into()))
    }

    fn emit_option_none(&mut self) -> EncoderResult {
        bail!(ErrorKind::UnsupportedDataType("option none".into()))
    }

    fn emit_option_some<F>(&mut self, _: F) -> EncoderResult
        where F: FnOnce(&mut Self) -> EncoderResult
    {
        bail!(ErrorKind::UnsupportedDataType("option some".into()))
    }

    fn emit_seq<F>(&mut self, len: usize, f: F) -> EncoderResult
        where F: FnOnce(&mut Self) -> EncoderResult
    {
        self.output.push(Vec::new());
        let position = self.output.len();
        self.write_size(len);
        f(self).chain_err(|| ErrorKind::UnsupportedDataType("array".into()))?;
        let length = self.output[position..].iter().map(|v| v.len()).sum();
        Ok(self.write_size_in_middle(position - 1, length))
    }

    fn emit_seq_elt<F>(&mut self, _: usize, f: F) -> EncoderResult
        where F: FnOnce(&mut Self) -> EncoderResult
    {
        f(self)
    }

    fn emit_map<F>(&mut self, _: usize, _: F) -> EncoderResult
        where F: FnOnce(&mut Self) -> EncoderResult
    {
        bail!(ErrorKind::UnsupportedDataType("map".into()))
    }

    fn emit_map_elt_key<F>(&mut self, _: usize, _: F) -> EncoderResult
        where F: FnOnce(&mut Self) -> EncoderResult
    {
        bail!(ErrorKind::UnsupportedDataType("map element key".into()))
    }

    fn emit_map_elt_val<F>(&mut self, _: usize, _: F) -> EncoderResult
        where F: FnOnce(&mut Self) -> EncoderResult
    {
        bail!(ErrorKind::UnsupportedDataType("map element value".into()))
    }
}

pub mod error {
    error_chain! {
        errors {
            UnsupportedDataType(t: String) {
                description("Datatype is not supported")
                display("Datatype is not supported, issue within {}", t)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std;
    use rustc_serialize::Encodable;

    fn pull_data(encoder: &Encoder) -> Vec<u8> {
        let mut cursor = std::io::Cursor::new(Vec::new());
        encoder.write_to(&mut cursor).unwrap();
        cursor.into_inner()
    }

    #[test]
    fn starts_empty() {
        assert_eq!(0, Encoder::new().len());
    }

    #[test]
    fn writes_u8() {
        let mut encoder = Encoder::new();
        150u8.encode(&mut encoder).unwrap();
        assert_eq!(vec![150], pull_data(&encoder));
    }

    #[test]
    fn writes_u16() {
        let mut encoder = Encoder::new();
        0xA234u16.encode(&mut encoder).unwrap();
        assert_eq!(vec![0x34, 0xA2], pull_data(&encoder));
    }

    #[test]
    fn writes_u32() {
        let mut encoder = Encoder::new();
        0xCD012345u32.encode(&mut encoder).unwrap();
        assert_eq!(vec![0x45, 0x23, 1, 0xCD], pull_data(&encoder));
    }

    #[test]
    fn writes_u64() {
        let mut encoder = Encoder::new();
        0xAB9876543210AABBu64.encode(&mut encoder).unwrap();
        assert_eq!(vec![0xBB, 0xAA, 0x10, 0x32, 0x54, 0x76, 0x98, 0xAB],
                   pull_data(&encoder));
    }

    #[test]
    fn writes_i8() {
        let mut encoder = Encoder::new();
        (-100i8).encode(&mut encoder).unwrap();
        assert_eq!(vec![156], pull_data(&encoder));
    }

    #[test]
    fn writes_i16() {
        let mut encoder = Encoder::new();
        (-30000i16).encode(&mut encoder).unwrap();
        assert_eq!(vec![0xD0, 0x8A], pull_data(&encoder));
    }

    #[test]
    fn writes_i32() {
        let mut encoder = Encoder::new();
        (-2000000000i32).encode(&mut encoder).unwrap();
        assert_eq!(vec![0x00, 0x6C, 0xCA, 0x88], pull_data(&encoder));
    }

    #[test]
    fn writes_i64() {
        let mut encoder = Encoder::new();
        (-9000000000000000000i64).encode(&mut encoder).unwrap();
        assert_eq!(vec![0x00, 0x00, 0x7c, 0x1d, 0xaf, 0x93, 0x19, 0x83],
                   pull_data(&encoder));
    }

    #[test]
    fn writes_f32() {
        let mut encoder = Encoder::new();
        (1005.75f32).encode(&mut encoder).unwrap();
        assert_eq!(vec![0x00, 0x70, 0x7b, 0x44], pull_data(&encoder));
    }

    #[test]
    fn writes_f64() {
        let mut encoder = Encoder::new();
        (1005.75f64).encode(&mut encoder).unwrap();
        assert_eq!(vec![0x00, 0x00, 0x00, 0x00, 0x00, 0x6e, 0x8f, 0x40],
                   pull_data(&encoder));
    }

    #[test]
    fn writes_bool() {
        let mut encoder = Encoder::new();
        true.encode(&mut encoder).unwrap();
        assert_eq!(vec![1], pull_data(&encoder));
        let mut encoder = Encoder::new();
        false.encode(&mut encoder).unwrap();
        assert_eq!(vec![0], pull_data(&encoder));
    }

    #[test]
    fn writes_string() {
        let mut encoder = Encoder::new();
        "".encode(&mut encoder).unwrap();
        assert_eq!(vec![0, 0, 0, 0], pull_data(&encoder));
        let mut encoder = Encoder::new();
        "Hello, World!".encode(&mut encoder).unwrap();
        assert_eq!(vec![13, 0, 0, 0, 72, 101, 108, 108, 111, 44, 32, 87, 111, 114, 108, 100, 33],
                   pull_data(&encoder));
    }

    #[test]
    fn writes_array() {
        let mut encoder = Encoder::new();
        [7i16, 1025, 33, 57].encode(&mut encoder).unwrap();
        assert_eq!(vec![12, 0, 0, 0, 4, 0, 0, 0, 7, 0, 1, 4, 33, 0, 57, 0],
                   pull_data(&encoder));
    }

    #[test]
    fn writes_tuple() {
        let mut encoder = Encoder::new();
        (2050i16, true, 7u8, "ABC012", vec![true, false, false, true])
            .encode(&mut encoder)
            .unwrap();
        assert_eq!(vec![26, 0, 0, 0, 2, 8, 1, 7, 6, 0, 0, 0, 65, 66, 67, 48, 49, 50, 8, 0, 0, 0,
                        4, 0, 0, 0, 1, 0, 0, 1],
                   pull_data(&encoder));
    }

    #[derive(RustcEncodable)]
    struct TestStructOne {
        a: i16,
        b: bool,
        c: u8,
        d: String,
        e: Vec<bool>,
    }

    #[test]
    fn writes_simple_struct() {
        let mut encoder = Encoder::new();
        TestStructOne {
                a: 2050i16,
                b: true,
                c: 7u8,
                d: String::from("ABC012"),
                e: vec![true, false, false, true],
            }
            .encode(&mut encoder)
            .unwrap();
        assert_eq!(vec![26, 0, 0, 0, 2, 8, 1, 7, 6, 0, 0, 0, 65, 66, 67, 48, 49, 50, 8, 0, 0, 0,
                        4, 0, 0, 0, 1, 0, 0, 1],
                   pull_data(&encoder));
    }

    #[derive(RustcEncodable)]
    struct TestStructPart {
        a: String,
        b: bool,
    }

    #[derive(RustcEncodable)]
    struct TestStructBig {
        a: Vec<TestStructPart>,
        b: String,
    }

    #[test]
    fn writes_complex_struct() {
        let mut encoder = Encoder::new();
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
        TestStructBig {
                a: parts,
                b: String::from("EEe"),
            }
            .encode(&mut encoder)
            .unwrap();
        assert_eq!(vec![54, 0, 0, 0, 43, 0, 0, 0, 3, 0, 0, 0, 8, 0, 0, 0, 3, 0, 0, 0, 65, 66, 67,
                        1, 10, 0, 0, 0, 5, 0, 0, 0, 49, 33, 33, 33, 33, 1, 9, 0, 0, 0, 4, 0, 0,
                        0, 50, 51, 52, 98, 0, 3, 0, 0, 0, 69, 69, 101],
                   pull_data(&encoder));
        assert_eq!(58, encoder.len());
    }
}
