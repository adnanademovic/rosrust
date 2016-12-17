use rustc_serialize;
use std;
use std::error::Error as ErrorTrait;
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

    pub fn new_request<T: std::io::Read>(body: T)
                                         -> Result<(String, Vec<Decoder>), value::DecodeError> {
        let value = value::XmlRpcRequest::new(body)?;
        Ok((value.method, value.parameters.into_iter().map(Decoder::new).collect()))
    }

    pub fn new_response<T: std::io::Read>(body: T) -> Result<Vec<Decoder>, value::DecodeError> {
        let value = value::XmlRpcResponse::new(body)?;
        Ok(value.parameters.into_iter().map(Decoder::new).collect())
    }

    pub fn value(self) -> value::XmlRpcValue {
        self.value
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
    type Error = Error;

    fn read_nil(&mut self) -> Result<(), Self::Error> {
        Err(Error::UnsupportedDataFormat)
    }

    fn read_usize(&mut self) -> Result<usize, Self::Error> {
        Err(Error::UnsupportedDataFormat)
    }

    fn read_u64(&mut self) -> Result<u64, Self::Error> {
        Err(Error::UnsupportedDataFormat)
    }

    fn read_u32(&mut self) -> Result<u32, Self::Error> {
        Err(Error::UnsupportedDataFormat)
    }

    fn read_u16(&mut self) -> Result<u16, Self::Error> {
        Err(Error::UnsupportedDataFormat)
    }

    fn read_u8(&mut self) -> Result<u8, Self::Error> {
        Err(Error::UnsupportedDataFormat)
    }

    fn read_isize(&mut self) -> Result<isize, Self::Error> {
        Err(Error::UnsupportedDataFormat)
    }

    fn read_i64(&mut self) -> Result<i64, Self::Error> {
        Err(Error::UnsupportedDataFormat)
    }

    fn read_i32(&mut self) -> Result<i32, Self::Error> {
        if let Some((value::XmlRpcValue::Int(v), _)) = self.chain.next() {
            Ok(v)
        } else {
            Err(Error::MismatchedDataFormat)
        }
    }

    fn read_i16(&mut self) -> Result<i16, Self::Error> {
        Err(Error::UnsupportedDataFormat)
    }

    fn read_i8(&mut self) -> Result<i8, Self::Error> {
        Err(Error::UnsupportedDataFormat)
    }

    fn read_bool(&mut self) -> Result<bool, Self::Error> {
        if let Some((value::XmlRpcValue::Bool(v), _)) = self.chain.next() {
            Ok(v)
        } else {
            Err(Error::MismatchedDataFormat)
        }
    }

    fn read_f64(&mut self) -> Result<f64, Self::Error> {
        if let Some((value::XmlRpcValue::Double(v), _)) = self.chain.next() {
            Ok(v)
        } else {
            Err(Error::MismatchedDataFormat)
        }
    }

    fn read_f32(&mut self) -> Result<f32, Self::Error> {
        Err(Error::UnsupportedDataFormat)
    }

    fn read_char(&mut self) -> Result<char, Self::Error> {
        Err(Error::UnsupportedDataFormat)
    }

    fn read_str(&mut self) -> Result<String, Self::Error> {
        if let Some((value::XmlRpcValue::String(v), _)) = self.chain.next() {
            Ok(v)
        } else {
            Err(Error::MismatchedDataFormat)
        }
    }

    fn read_enum<T, F>(&mut self, _: &str, _: F) -> Result<T, Self::Error>
        where F: FnOnce(&mut Self) -> Result<T, Self::Error>
    {
        Err(Error::UnsupportedDataFormat)
    }

    fn read_enum_variant<T, F>(&mut self, _: &[&str], _: F) -> Result<T, Self::Error>
        where F: FnMut(&mut Self, usize) -> Result<T, Self::Error>
    {
        Err(Error::UnsupportedDataFormat)
    }

    fn read_enum_variant_arg<T, F>(&mut self, _: usize, _: F) -> Result<T, Self::Error>
        where F: FnOnce(&mut Self) -> Result<T, Self::Error>
    {
        Err(Error::UnsupportedDataFormat)
    }

    fn read_enum_struct_variant<T, F>(&mut self, _: &[&str], _: F) -> Result<T, Self::Error>
        where F: FnMut(&mut Self, usize) -> Result<T, Self::Error>
    {
        Err(Error::UnsupportedDataFormat)
    }

    fn read_enum_struct_variant_field<T, F>(&mut self,
                                            _: &str,
                                            _: usize,
                                            _: F)
                                            -> Result<T, Self::Error>
        where F: FnOnce(&mut Self) -> Result<T, Self::Error>
    {
        Err(Error::UnsupportedDataFormat)
    }

    fn read_struct<T, F>(&mut self, _: &str, size: usize, f: F) -> Result<T, Self::Error>
        where F: FnOnce(&mut Self) -> Result<T, Self::Error>
    {
        self.read_tuple(size, f)
    }

    fn read_struct_field<T, F>(&mut self, _: &str, _: usize, f: F) -> Result<T, Self::Error>
        where F: FnOnce(&mut Self) -> Result<T, Self::Error>
    {
        f(self)
    }

    fn read_tuple<T, F>(&mut self, l: usize, f: F) -> Result<T, Self::Error>
        where F: FnOnce(&mut Self) -> Result<T, Self::Error>
    {
        if let Some((value::XmlRpcValue::Array(_), len)) = self.chain.next() {
            if l == len {
                f(self)
            } else {
                Err(Error::MismatchedDataFormat)
            }
        } else {
            Err(Error::MismatchedDataFormat)
        }
    }

    fn read_tuple_arg<T, F>(&mut self, _: usize, f: F) -> Result<T, Self::Error>
        where F: FnOnce(&mut Self) -> Result<T, Self::Error>
    {
        f(self)
    }

    fn read_tuple_struct<T, F>(&mut self, _: &str, size: usize, f: F) -> Result<T, Self::Error>
        where F: FnOnce(&mut Self) -> Result<T, Self::Error>
    {
        self.read_tuple(size, f)
    }

    fn read_tuple_struct_arg<T, F>(&mut self, _: usize, f: F) -> Result<T, Self::Error>
        where F: FnOnce(&mut Self) -> Result<T, Self::Error>
    {
        f(self)
    }

    fn read_option<T, F>(&mut self, _: F) -> Result<T, Self::Error>
        where F: FnMut(&mut Self, bool) -> Result<T, Self::Error>
    {
        Err(Error::UnsupportedDataFormat)
    }

    fn read_seq<T, F>(&mut self, f: F) -> Result<T, Self::Error>
        where F: FnOnce(&mut Self, usize) -> Result<T, Self::Error>
    {
        if let Some((value::XmlRpcValue::Array(_), len)) = self.chain.next() {
            f(self, len)
        } else {
            Err(Error::MismatchedDataFormat)
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
        Err(Error::UnsupportedDataFormat)
    }

    fn read_map_elt_key<T, F>(&mut self, _: usize, _: F) -> Result<T, Self::Error>
        where F: FnOnce(&mut Self) -> Result<T, Self::Error>
    {
        Err(Error::UnsupportedDataFormat)
    }

    fn read_map_elt_val<T, F>(&mut self, _: usize, _: F) -> Result<T, Self::Error>
        where F: FnOnce(&mut Self) -> Result<T, Self::Error>
    {
        Err(Error::UnsupportedDataFormat)
    }

    fn error(&mut self, s: &str) -> Self::Error {
        Error::Other(String::from(s))
    }
}

#[derive(Debug)]
pub enum Error {
    Other(String),
    MismatchedDataFormat,
    UnsupportedDataFormat,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.description())
    }
}

impl std::error::Error for Error {
    fn description(&self) -> &str {
        match *self {
            Error::Other(ref err) => err,
            Error::MismatchedDataFormat => "Given XML-RPC value tree does not match target format",
            Error::UnsupportedDataFormat => {
                "Decoder does not support all members of given data structure"
            }
        }
    }

    fn cause(&self) -> Option<&std::error::Error> {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::XmlRpcValue;
    use rustc_serialize::Decodable;
    use std;

    #[test]
    fn reads_string() {
        assert_eq!("First test", String::decode(
            &mut Decoder::new(XmlRpcValue::String(String::from("First test")))).unwrap());
    }

    #[test]
    fn reads_int() {
        assert_eq!(41,
                   i32::decode(&mut Decoder::new(XmlRpcValue::Int(41))).unwrap());
    }

    #[test]
    fn reads_float() {
        assert_eq!(32.5,
                   f64::decode(&mut Decoder::new(XmlRpcValue::Double(32.5))).unwrap());
    }

    #[test]
    fn reads_bool() {
        assert_eq!(true,
                   bool::decode(&mut Decoder::new(XmlRpcValue::Bool(true))).unwrap());
        assert_eq!(false,
                   bool::decode(&mut Decoder::new(XmlRpcValue::Bool(false))).unwrap());
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
                       .unwrap());
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
                       .unwrap());
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
                       .unwrap());
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
        let (method, mut parameters) = Decoder::new_request(&mut cursor).unwrap();
        assert_eq!("mytype.mymethod", method);
        assert_eq!(2, parameters.len());
        assert_eq!(33, i32::decode(&mut parameters[0]).unwrap());
        assert_eq!(ExampleRequestStruct {
                       a: 41,
                       b: true,
                       c: ExampleRequestStructChild {
                           a: String::from("Hello"),
                           b: 0.5,
                       },
                   },
                   ExampleRequestStruct::decode(&mut parameters[1]).unwrap());
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
        let mut parameters = Decoder::new_response(&mut cursor).unwrap();
        assert_eq!(2, parameters.len());
        assert_eq!(33, i32::decode(&mut parameters[0]).unwrap());
        assert_eq!(ExampleRequestStruct {
                       a: 41,
                       b: true,
                       c: ExampleRequestStructChild {
                           a: String::from("Hello"),
                           b: 0.5,
                       },
                   },
                   ExampleRequestStruct::decode(&mut parameters[1]).unwrap());
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
        let (method, mut parameters) = Decoder::new_request(&mut cursor).unwrap();
        assert_eq!("mytype.mymethod", method);
        assert_eq!(2, parameters.len());
        assert_eq!(XmlRpcValue::Array(vec![
            XmlRpcValue::Int(41),
            XmlRpcValue::Bool(true),
            XmlRpcValue::Array(vec![
                XmlRpcValue::String(String::from("Hello")),
                XmlRpcValue::Double(0.5),
            ]),
        ]),
                   parameters.pop().unwrap().value());
        assert_eq!(XmlRpcValue::Int(33), parameters.pop().unwrap().value());
    }
}
