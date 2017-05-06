use rustc_serialize;
use std;
use super::error::{ErrorKind, ResultExt, Result};
use super::value;

pub struct Decoder {
    value: value::XmlRpcValue,
    chain: std::vec::IntoIter<(value::XmlRpcValue, usize)>,
}

impl Decoder {
    pub fn new(body: value::XmlRpcValue) -> Decoder {
        let mut chain = vec![];
        append_elements(&mut chain, &body);
        Decoder {
            value: body,
            chain: chain.into_iter(),
        }
    }

    pub fn new_request<T: std::io::Read>(body: T) -> Result<(String, Vec<Decoder>)> {
        value::XmlRpcRequest::new(body)
            .chain_err(|| ErrorKind::XmlRpcReading("request".into()))
            .map(|value| {
                     (value.method,
                      value
                          .parameters
                          .into_iter()
                          .map(Decoder::new)
                          .collect())
                 })
    }

    pub fn new_response<T: std::io::Read>(body: T) -> Result<Vec<Decoder>> {
        value::XmlRpcResponse::new(body)
            .chain_err(|| ErrorKind::XmlRpcReading("response".into()))
            .map(|value| {
                     value
                         .parameters
                         .into_iter()
                         .map(Decoder::new)
                         .collect()
                 })
    }

    pub fn value(self) -> value::XmlRpcValue {
        self.value
    }

    fn read_tuple_helper<T, F>(&mut self, l: usize, f: F) -> Result<T>
        where F: FnOnce(&mut Self) -> Result<T>
    {
        if let Some((value::XmlRpcValue::Array(_), len)) = self.chain.next() {
            if l == len {
                f(self)
            } else {
                Err(ErrorKind::MismatchedDataFormat(format!("an array of length {}", len)).into())
            }
        } else {
            Err(ErrorKind::MismatchedDataFormat("an array field".into()).into())
        }
    }
}

fn append_elements(array: &mut Vec<(value::XmlRpcValue, usize)>, v: &value::XmlRpcValue) {
    if let &value::XmlRpcValue::Array(ref children) = v {
        array.push((value::XmlRpcValue::Array(vec![]), children.len()));
        for child in children {
            append_elements(array, child);
        }
    } else {
        array.push((v.clone(), 0));
    }
}

impl rustc_serialize::Decoder for Decoder {
    type Error = super::error::Error;

    fn read_nil(&mut self) -> Result<()> {
        bail!(ErrorKind::UnsupportedDataType("nil".into()))
    }

    fn read_usize(&mut self) -> Result<usize> {
        bail!(ErrorKind::UnsupportedDataType("usize".into()))
    }

    fn read_u64(&mut self) -> Result<u64> {
        bail!(ErrorKind::UnsupportedDataType("u64".into()))
    }

    fn read_u32(&mut self) -> Result<u32> {
        bail!(ErrorKind::UnsupportedDataType("u32".into()))
    }

    fn read_u16(&mut self) -> Result<u16> {
        bail!(ErrorKind::UnsupportedDataType("u16".into()))
    }

    fn read_u8(&mut self) -> Result<u8> {
        bail!(ErrorKind::UnsupportedDataType("u8".into()))
    }

    fn read_isize(&mut self) -> Result<isize> {
        bail!(ErrorKind::UnsupportedDataType("isize".into()))
    }

    fn read_i64(&mut self) -> Result<i64> {
        bail!(ErrorKind::UnsupportedDataType("i64".into()))
    }

    fn read_i32(&mut self) -> Result<i32> {
        if let Some((value::XmlRpcValue::Int(v), _)) = self.chain.next() {
            Ok(v)
        } else {
            Err(ErrorKind::MismatchedDataFormat("an integer (i32) field".into()).into())
        }
    }

    fn read_i16(&mut self) -> Result<i16> {
        bail!(ErrorKind::UnsupportedDataType("i16".into()))
    }

    fn read_i8(&mut self) -> Result<i8> {
        bail!(ErrorKind::UnsupportedDataType("i8".into()))
    }

    fn read_bool(&mut self) -> Result<bool> {
        if let Some((value::XmlRpcValue::Bool(v), _)) = self.chain.next() {
            Ok(v)
        } else {
            Err(ErrorKind::MismatchedDataFormat("a boolean field".into()).into())
        }
    }

    fn read_f64(&mut self) -> Result<f64> {
        if let Some((value::XmlRpcValue::Double(v), _)) = self.chain.next() {
            Ok(v)
        } else {
            Err(ErrorKind::MismatchedDataFormat("a double (f64) field".into()).into())
        }
    }

    fn read_f32(&mut self) -> Result<f32> {
        bail!(ErrorKind::UnsupportedDataType("f32".into()))
    }

    fn read_char(&mut self) -> Result<char> {
        bail!(ErrorKind::UnsupportedDataType("char".into()))
    }

    fn read_str(&mut self) -> Result<String> {
        if let Some((value::XmlRpcValue::String(v), _)) = self.chain.next() {
            Ok(v)
        } else {
            Err(ErrorKind::MismatchedDataFormat("a string field".into()).into())
        }
    }

    fn read_enum<T, F>(&mut self, _: &str, _: F) -> Result<T>
        where F: FnOnce(&mut Self) -> Result<T>
    {
        bail!(ErrorKind::UnsupportedDataType("enum".into()))
    }

    fn read_enum_variant<T, F>(&mut self, _: &[&str], _: F) -> Result<T>
        where F: FnMut(&mut Self, usize) -> Result<T>
    {
        bail!(ErrorKind::UnsupportedDataType("enum variant".into()))
    }

    fn read_enum_variant_arg<T, F>(&mut self, _: usize, _: F) -> Result<T>
        where F: FnOnce(&mut Self) -> Result<T>
    {
        bail!(ErrorKind::UnsupportedDataType("enum variant argument".into()))
    }

    fn read_enum_struct_variant<T, F>(&mut self, _: &[&str], _: F) -> Result<T>
        where F: FnMut(&mut Self, usize) -> Result<T>
    {
        bail!(ErrorKind::UnsupportedDataType("enum struct variant".into()))
    }

    fn read_enum_struct_variant_field<T, F>(&mut self, _: &str, _: usize, _: F) -> Result<T>
        where F: FnOnce(&mut Self) -> Result<T>
    {
        bail!(ErrorKind::UnsupportedDataType("enum struct variant field".into()))
    }

    fn read_struct<T, F>(&mut self, name: &str, size: usize, f: F) -> Result<T>
        where F: FnOnce(&mut Self) -> Result<T>
    {
        self.read_tuple_helper(size, f)
            .chain_err(|| ErrorKind::Decoding(format!("struct {}", name)))
    }

    fn read_struct_field<T, F>(&mut self, name: &str, _: usize, f: F) -> Result<T>
        where F: FnOnce(&mut Self) -> Result<T>
    {
        f(self).chain_err(|| ErrorKind::Decoding(format!("field {}", name)))
    }

    fn read_tuple<T, F>(&mut self, size: usize, f: F) -> Result<T>
        where F: FnOnce(&mut Self) -> Result<T>
    {
        self.read_tuple_helper(size, f)
            .chain_err(|| ErrorKind::Decoding("tuple".into()))
    }

    fn read_tuple_arg<T, F>(&mut self, n: usize, f: F) -> Result<T>
        where F: FnOnce(&mut Self) -> Result<T>
    {
        f(self).chain_err(|| ErrorKind::Decoding(format!("field number {}", n)))
    }

    fn read_tuple_struct<T, F>(&mut self, name: &str, size: usize, f: F) -> Result<T>
        where F: FnOnce(&mut Self) -> Result<T>
    {
        self.read_tuple_helper(size, f)
            .chain_err(|| ErrorKind::Decoding(format!("tuple struct {}", name)))
    }

    fn read_tuple_struct_arg<T, F>(&mut self, n: usize, f: F) -> Result<T>
        where F: FnOnce(&mut Self) -> Result<T>
    {
        f(self).chain_err(|| ErrorKind::Decoding(format!("field number {}", n)))
    }

    fn read_option<T, F>(&mut self, _: F) -> Result<T>
        where F: FnMut(&mut Self, bool) -> Result<T>
    {
        bail!(ErrorKind::UnsupportedDataType("option".into()))
    }

    fn read_seq<T, F>(&mut self, f: F) -> Result<T>
        where F: FnOnce(&mut Self, usize) -> Result<T>
    {
        if let Some((value::XmlRpcValue::Array(_), len)) = self.chain.next() {
            f(self, len).chain_err(|| ErrorKind::Decoding("array".into()))
        } else {
            Err(ErrorKind::MismatchedDataFormat("an array field".into()).into())
        }
    }

    fn read_seq_elt<T, F>(&mut self, n: usize, f: F) -> Result<T>
        where F: FnOnce(&mut Self) -> Result<T>
    {
        f(self).chain_err(|| ErrorKind::Decoding(format!("element number {}", n)))
    }

    fn read_map<T, F>(&mut self, _: F) -> Result<T>
        where F: FnOnce(&mut Self, usize) -> Result<T>
    {
        bail!(ErrorKind::UnsupportedDataType("map".into()))
    }

    fn read_map_elt_key<T, F>(&mut self, _: usize, _: F) -> Result<T>
        where F: FnOnce(&mut Self) -> Result<T>
    {
        bail!(ErrorKind::UnsupportedDataType("map element key".into()))
    }

    fn read_map_elt_val<T, F>(&mut self, _: usize, _: F) -> Result<T>
        where F: FnOnce(&mut Self) -> Result<T>
    {
        bail!(ErrorKind::UnsupportedDataType("map element value".into()))
    }

    fn error(&mut self, s: &str) -> Self::Error {
        s.into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::XmlRpcValue;
    use rustc_serialize::Decodable;
    use std;

    static FAILED_TO_DECODE: &'static str = "Failed to decode";

    #[test]
    fn reads_string() {
        assert_eq!("First test", String::decode(
            &mut Decoder::new(XmlRpcValue::String(String::from("First test"))))
                   .expect(FAILED_TO_DECODE));
    }

    #[test]
    fn reads_int() {
        assert_eq!(41,
                   i32::decode(&mut Decoder::new(XmlRpcValue::Int(41))).expect(FAILED_TO_DECODE));
    }

    #[test]
    fn reads_float() {
        assert_eq!(32.5,
                   f64::decode(&mut Decoder::new(XmlRpcValue::Double(32.5)))
                       .expect(FAILED_TO_DECODE));
    }

    #[test]
    fn reads_bool() {
        assert_eq!(true,
                   bool::decode(&mut Decoder::new(XmlRpcValue::Bool(true)))
                       .expect(FAILED_TO_DECODE));
        assert_eq!(false,
                   bool::decode(&mut Decoder::new(XmlRpcValue::Bool(false)))
                       .expect(FAILED_TO_DECODE));
    }

    #[test]
    fn reads_array() {
        assert_eq!(vec![1, 2, 3, 4, 5],
                   Vec::<i32>::decode(&mut Decoder::new(XmlRpcValue::Array(vec![
                       XmlRpcValue::Int(1),
                       XmlRpcValue::Int(2),
                       XmlRpcValue::Int(3),
                       XmlRpcValue::Int(4),
                       XmlRpcValue::Int(5),
                   ])))
                           .expect(FAILED_TO_DECODE));
    }

    #[derive(Debug,PartialEq,RustcDecodable)]
    struct ExampleTuple(i32, f64, String, bool);

    #[test]
    fn reads_tuple() {
        assert_eq!(ExampleTuple(5, 0.5, String::from("abc"), false),
                   ExampleTuple::decode(&mut Decoder::new(XmlRpcValue::Array(vec![
                       XmlRpcValue::Int(5),
                       XmlRpcValue::Double(0.5),
                       XmlRpcValue::String(String::from("abc")),
                       XmlRpcValue::Bool(false),
                   ])))
                           .expect(FAILED_TO_DECODE));
    }

    #[derive(Debug,PartialEq,RustcDecodable)]
    struct ExampleStruct {
        a: i32,
        b: ExampleTuple,
    }

    #[test]
    fn reads_struct() {
        assert_eq!(ExampleStruct {
                       a: 11,
                       b: ExampleTuple(5, 0.5, String::from("abc"), false),
                   },
                   ExampleStruct::decode(&mut Decoder::new(XmlRpcValue::Array(vec![
                       XmlRpcValue::Int(11),
                       XmlRpcValue::Array(vec![
                           XmlRpcValue::Int(5),
                           XmlRpcValue::Double(0.5),
                           XmlRpcValue::String(String::from("abc")),
                           XmlRpcValue::Bool(false),
                       ]),
                   ])))
                           .expect(FAILED_TO_DECODE));
    }

    #[derive(Debug,PartialEq,RustcDecodable)]
    struct ExampleRequestStruct {
        a: i32,
        b: bool,
        c: ExampleRequestStructChild,
    }

    #[derive(Debug,PartialEq,RustcDecodable)]
    struct ExampleRequestStructChild {
        a: String,
        b: f64,
    }

    #[test]
    fn handles_requests() {
        let data = r#"<?xml version="1.0"?>
<methodCall>
  <methodName>mytype.mymethod</methodName>
  <params>
    <param>
      <value><i4>33</i4></value>
    </param>
    <param>
      <value><array><data>
        <value><i4>41</i4></value>
        <value><boolean>1</boolean></value>
        <value><array><data>
          <value><string>Hello</string></value>
          <value><double>0.5</double></value>
        </data></array></value>
      </data></array></value>
    </param>
  </params>
</methodCall>"#;
        let mut cursor = std::io::Cursor::new(data.as_bytes());
        let (method, mut parameters) = Decoder::new_request(&mut cursor).expect(FAILED_TO_DECODE);
        assert_eq!("mytype.mymethod", method);
        assert_eq!(2, parameters.len());
        assert_eq!(33, i32::decode(&mut parameters[0]).expect(FAILED_TO_DECODE));
        assert_eq!(ExampleRequestStruct {
                       a: 41,
                       b: true,
                       c: ExampleRequestStructChild {
                           a: String::from("Hello"),
                           b: 0.5,
                       },
                   },
                   ExampleRequestStruct::decode(&mut parameters[1]).expect(FAILED_TO_DECODE));
    }

    #[test]
    fn handles_responses() {
        let data = r#"<?xml version="1.0"?>
<methodResponse>
  <params>
    <param>
      <value><i4>33</i4></value>
    </param>
    <param>
      <value><array><data>
        <value><i4>41</i4></value>
        <value><boolean>1</boolean></value>
        <value><array><data>
          <value><string>Hello</string></value>
          <value><double>0.5</double></value>
        </data></array></value>
      </data></array></value>
    </param>
  </params>
</methodResponse>"#;
        let mut cursor = std::io::Cursor::new(data.as_bytes());
        let mut parameters = Decoder::new_response(&mut cursor).expect(FAILED_TO_DECODE);
        assert_eq!(2, parameters.len());
        assert_eq!(33, i32::decode(&mut parameters[0]).expect(FAILED_TO_DECODE));
        assert_eq!(ExampleRequestStruct {
                       a: 41,
                       b: true,
                       c: ExampleRequestStructChild {
                           a: String::from("Hello"),
                           b: 0.5,
                       },
                   },
                   ExampleRequestStruct::decode(&mut parameters[1]).expect(FAILED_TO_DECODE));
    }

    #[test]
    fn decoders_value_field_matches_data() {
        let data = r#"<?xml version="1.0"?>
<methodCall>
  <methodName>mytype.mymethod</methodName>
  <params>
    <param>
      <value><i4>33</i4></value>
    </param>
    <param>
      <value><array><data>
        <value><i4>41</i4></value>
        <value><boolean>1</boolean></value>
        <value><array><data>
          <value><string>Hello</string></value>
          <value><double>0.5</double></value>
        </data></array></value>
      </data></array></value>
    </param>
  </params>
</methodCall>"#;
        let mut cursor = std::io::Cursor::new(data.as_bytes());
        let (method, mut parameters) = Decoder::new_request(&mut cursor).expect(FAILED_TO_DECODE);
        assert_eq!("mytype.mymethod", method);
        assert_eq!(2, parameters.len());
        assert_eq!(XmlRpcValue::Array(vec![XmlRpcValue::Int(41),
                                           XmlRpcValue::Bool(true),
                                           XmlRpcValue::Array(vec![
                XmlRpcValue::String(String::from("Hello")),
                XmlRpcValue::Double(0.5),
            ])]),
                   parameters.pop().expect(FAILED_TO_DECODE).value());
        assert_eq!(XmlRpcValue::Int(33),
                   parameters.pop().expect(FAILED_TO_DECODE).value());
    }
}
