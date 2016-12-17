use rustc_serialize;
use std;
use std::error::Error as ErrorTrait;
use super::value::{self, XmlRpcValue};

pub struct Encoder {
    data: Vec<(XmlRpcValue, usize)>,
}

impl Encoder {
    pub fn new() -> Encoder {
        Encoder { data: vec![] }
    }

    fn form_tree(self) -> Vec<XmlRpcValue> {
        let mut retval = vec![];
        let mut data = self.data.into_iter();
        while let Some(value) = Encoder::form_subtree(&mut data) {
            retval.push(value);
        }
        retval
    }

    fn form_subtree(mut data: &mut std::vec::IntoIter<(XmlRpcValue, usize)>)
                    -> Option<XmlRpcValue> {
        match data.next() {
            Some((node, count)) => {
                Some(match node {
                    XmlRpcValue::Array(_) => {
                        XmlRpcValue::Array((0..count)
                            .into_iter()
                            .filter_map(|_| Encoder::form_subtree(&mut data))
                            .collect())
                    }
                    _ => node,
                })
            }
            None => None,
        }
    }

    pub fn write_request<T: std::io::Write>(self,
                                            method: &str,
                                            mut body: &mut T)
                                            -> Result<(), std::io::Error> {
        write!(&mut body,
               "{}",
               value::XmlRpcRequest {
                   method: String::from(method),
                   parameters: self.form_tree(),
               })
    }

    pub fn write_response<T: std::io::Write>(self, mut body: &mut T) -> Result<(), std::io::Error> {
        write!(&mut body,
               "{}",
               value::XmlRpcResponse { parameters: self.form_tree() })
    }
}

impl rustc_serialize::Encoder for Encoder {
    type Error = Error;

    fn emit_nil(&mut self) -> Result<(), Self::Error> {
        Err(Error {})
    }

    fn emit_usize(&mut self, _: usize) -> Result<(), Self::Error> {
        Err(Error {})
    }

    fn emit_u64(&mut self, _: u64) -> Result<(), Self::Error> {
        Err(Error {})
    }

    fn emit_u32(&mut self, _: u32) -> Result<(), Self::Error> {
        Err(Error {})
    }

    fn emit_u16(&mut self, _: u16) -> Result<(), Self::Error> {
        Err(Error {})
    }

    fn emit_u8(&mut self, _: u8) -> Result<(), Self::Error> {
        Err(Error {})
    }

    fn emit_isize(&mut self, _: isize) -> Result<(), Self::Error> {
        Err(Error {})
    }

    fn emit_i64(&mut self, _: i64) -> Result<(), Self::Error> {
        Err(Error {})
    }

    fn emit_i32(&mut self, v: i32) -> Result<(), Self::Error> {
        Ok(self.data.push((XmlRpcValue::Int(v), 0)))
    }

    fn emit_i16(&mut self, _: i16) -> Result<(), Self::Error> {
        Err(Error {})
    }

    fn emit_i8(&mut self, _: i8) -> Result<(), Self::Error> {
        Err(Error {})
    }

    fn emit_bool(&mut self, v: bool) -> Result<(), Self::Error> {
        Ok(self.data.push((XmlRpcValue::Bool(v), 0)))
    }

    fn emit_f64(&mut self, v: f64) -> Result<(), Self::Error> {
        Ok(self.data.push((XmlRpcValue::Double(v), 0)))
    }

    fn emit_f32(&mut self, _: f32) -> Result<(), Self::Error> {
        Err(Error {})
    }

    fn emit_char(&mut self, _: char) -> Result<(), Self::Error> {
        Err(Error {})
    }

    fn emit_str(&mut self, v: &str) -> Result<(), Self::Error> {
        Ok(self.data.push((XmlRpcValue::String(String::from(v)), 0)))
    }

    fn emit_enum<F>(&mut self, _: &str, _: F) -> Result<(), Self::Error>
        where F: FnOnce(&mut Self) -> Result<(), Self::Error>
    {
        Err(Error {})
    }

    fn emit_enum_variant<F>(&mut self, _: &str, _: usize, _: usize, _: F) -> Result<(), Self::Error>
        where F: FnOnce(&mut Self) -> Result<(), Self::Error>
    {
        Err(Error {})
    }

    fn emit_enum_variant_arg<F>(&mut self, _: usize, _: F) -> Result<(), Self::Error>
        where F: FnOnce(&mut Self) -> Result<(), Self::Error>
    {
        Err(Error {})
    }

    fn emit_enum_struct_variant<F>(&mut self,
                                   _: &str,
                                   _: usize,
                                   _: usize,
                                   _: F)
                                   -> Result<(), Self::Error>
        where F: FnOnce(&mut Self) -> Result<(), Self::Error>
    {
        Err(Error {})
    }

    fn emit_enum_struct_variant_field<F>(&mut self,
                                         _: &str,
                                         _: usize,
                                         _: F)
                                         -> Result<(), Self::Error>
        where F: FnOnce(&mut Self) -> Result<(), Self::Error>
    {
        Err(Error {})
    }

    fn emit_struct<F>(&mut self, _: &str, l: usize, f: F) -> Result<(), Self::Error>
        where F: FnOnce(&mut Self) -> Result<(), Self::Error>
    {
        self.data.push((XmlRpcValue::Array(vec![]), l));
        f(self)
    }

    fn emit_struct_field<F>(&mut self, _: &str, _: usize, f: F) -> Result<(), Self::Error>
        where F: FnOnce(&mut Self) -> Result<(), Self::Error>
    {
        f(self)
    }

    fn emit_tuple<F>(&mut self, l: usize, f: F) -> Result<(), Self::Error>
        where F: FnOnce(&mut Self) -> Result<(), Self::Error>
    {
        self.data.push((XmlRpcValue::Array(vec![]), l));
        f(self)
    }

    fn emit_tuple_arg<F>(&mut self, _: usize, f: F) -> Result<(), Self::Error>
        where F: FnOnce(&mut Self) -> Result<(), Self::Error>
    {
        f(self)
    }

    fn emit_tuple_struct<F>(&mut self, _: &str, l: usize, f: F) -> Result<(), Self::Error>
        where F: FnOnce(&mut Self) -> Result<(), Self::Error>
    {
        self.data.push((XmlRpcValue::Array(vec![]), l));
        f(self)
    }

    fn emit_tuple_struct_arg<F>(&mut self, _: usize, f: F) -> Result<(), Self::Error>
        where F: FnOnce(&mut Self) -> Result<(), Self::Error>
    {
        f(self)
    }

    fn emit_option<F>(&mut self, _: F) -> Result<(), Self::Error>
        where F: FnOnce(&mut Self) -> Result<(), Self::Error>
    {
        Err(Error {})
    }

    fn emit_option_none(&mut self) -> Result<(), Self::Error> {
        Err(Error {})
    }

    fn emit_option_some<F>(&mut self, _: F) -> Result<(), Self::Error>
        where F: FnOnce(&mut Self) -> Result<(), Self::Error>
    {
        Err(Error {})
    }

    fn emit_seq<F>(&mut self, l: usize, f: F) -> Result<(), Self::Error>
        where F: FnOnce(&mut Self) -> Result<(), Self::Error>
    {
        self.data.push((XmlRpcValue::Array(vec![]), l));
        f(self)
    }

    fn emit_seq_elt<F>(&mut self, _: usize, f: F) -> Result<(), Self::Error>
        where F: FnOnce(&mut Self) -> Result<(), Self::Error>
    {
        f(self)
    }

    fn emit_map<F>(&mut self, _: usize, _: F) -> Result<(), Self::Error>
        where F: FnOnce(&mut Self) -> Result<(), Self::Error>
    {
        Err(Error {})
    }

    fn emit_map_elt_key<F>(&mut self, _: usize, _: F) -> Result<(), Self::Error>
        where F: FnOnce(&mut Self) -> Result<(), Self::Error>
    {
        Err(Error {})
    }

    fn emit_map_elt_val<F>(&mut self, _: usize, _: F) -> Result<(), Self::Error>
        where F: FnOnce(&mut Self) -> Result<(), Self::Error>
    {
        Err(Error {})
    }
}

#[derive(Debug)]
pub struct Error {}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.description())
    }
}

impl std::error::Error for Error {
    fn description(&self) -> &str {
        "Decoder does not support all members of given data structure"
    }

    fn cause(&self) -> Option<&std::error::Error> {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rustc_serialize::Encodable;
    use std;

    #[test]
    fn writes_response() {
        let mut data = vec![];
        let mut encoder = Encoder::new();
        String::from("Hello").encode(&mut encoder).unwrap();
        encoder.write_response(&mut data).unwrap();
        assert_eq!(concat!(r#"<?xml version="1.0"?>"#,
                           r#"<methodResponse>"#,
                           r#"<params>"#,
                           r#"<param>"#,
                           r#"<value><string>Hello</string></value>"#,
                           r#"</param>"#,
                           r#"</params>"#,
                           r#"</methodResponse>"#),
                   std::str::from_utf8(&data).unwrap());
    }

    #[test]
    fn writes_request() {
        let mut data = vec![];
        let mut encoder = Encoder::new();
        String::from("Hello").encode(&mut encoder).unwrap();
        encoder.write_request("something", &mut data).unwrap();
        assert_eq!(concat!(r#"<?xml version="1.0"?>"#,
                           r#"<methodCall>"#,
                           r#"<methodName>"#,
                           r#"something"#,
                           r#"</methodName>"#,
                           r#"<params>"#,
                           r#"<param>"#,
                           r#"<value><string>Hello</string></value>"#,
                           r#"</param>"#,
                           r#"</params>"#,
                           r#"</methodCall>"#),
                   std::str::from_utf8(&data).unwrap());
    }

    #[test]
    fn writes_string() {
        let mut data = vec![];
        let mut encoder = Encoder::new();
        String::from("Hello").encode(&mut encoder).unwrap();
        String::from("There").encode(&mut encoder).unwrap();
        String::from("Friend").encode(&mut encoder).unwrap();
        encoder.write_response(&mut data).unwrap();
        assert_eq!(concat!(r#"<?xml version="1.0"?>"#,
                           r#"<methodResponse>"#,
                           r#"<params>"#,
                           r#"<param>"#,
                           r#"<value><string>Hello</string></value>"#,
                           r#"</param>"#,
                           r#"<param>"#,
                           r#"<value><string>There</string></value>"#,
                           r#"</param>"#,
                           r#"<param>"#,
                           r#"<value><string>Friend</string></value>"#,
                           r#"</param>"#,
                           r#"</params>"#,
                           r#"</methodResponse>"#),
                   std::str::from_utf8(&data).unwrap());
    }

    #[test]
    fn writes_int() {
        let mut data = vec![];
        let mut encoder = Encoder::new();
        43i32.encode(&mut encoder).unwrap();
        27i32.encode(&mut encoder).unwrap();
        12i32.encode(&mut encoder).unwrap();
        encoder.write_response(&mut data).unwrap();
        assert_eq!(concat!(r#"<?xml version="1.0"?>"#,
                           r#"<methodResponse>"#,
                           r#"<params>"#,
                           r#"<param>"#,
                           r#"<value><i4>43</i4></value>"#,
                           r#"</param>"#,
                           r#"<param>"#,
                           r#"<value><i4>27</i4></value>"#,
                           r#"</param>"#,
                           r#"<param>"#,
                           r#"<value><i4>12</i4></value>"#,
                           r#"</param>"#,
                           r#"</params>"#,
                           r#"</methodResponse>"#),
                   std::str::from_utf8(&data).unwrap());
    }

    #[test]
    fn writes_float() {
        let mut data = vec![];
        let mut encoder = Encoder::new();
        33.5f64.encode(&mut encoder).unwrap();
        11.25f64.encode(&mut encoder).unwrap();
        77.125f64.encode(&mut encoder).unwrap();
        encoder.write_response(&mut data).unwrap();
        assert_eq!(concat!(r#"<?xml version="1.0"?>"#,
                           r#"<methodResponse>"#,
                           r#"<params>"#,
                           r#"<param>"#,
                           r#"<value><double>33.5</double></value>"#,
                           r#"</param>"#,
                           r#"<param>"#,
                           r#"<value><double>11.25</double></value>"#,
                           r#"</param>"#,
                           r#"<param>"#,
                           r#"<value><double>77.125</double></value>"#,
                           r#"</param>"#,
                           r#"</params>"#,
                           r#"</methodResponse>"#),
                   std::str::from_utf8(&data).unwrap());
    }

    #[test]
    fn writes_bool() {
        let mut data = vec![];
        let mut encoder = Encoder::new();
        true.encode(&mut encoder).unwrap();
        false.encode(&mut encoder).unwrap();
        encoder.write_response(&mut data).unwrap();
        assert_eq!(concat!(r#"<?xml version="1.0"?>"#,
                           r#"<methodResponse>"#,
                           r#"<params>"#,
                           r#"<param>"#,
                           r#"<value><boolean>1</boolean></value>"#,
                           r#"</param>"#,
                           r#"<param>"#,
                           r#"<value><boolean>0</boolean></value>"#,
                           r#"</param>"#,
                           r#"</params>"#,
                           r#"</methodResponse>"#),
                   std::str::from_utf8(&data).unwrap());
    }

    #[test]
    fn writes_array() {
        let mut data = vec![];
        let mut encoder = Encoder::new();
        vec![1i32, 2, 3, 4, 5].encode(&mut encoder).unwrap();
        encoder.write_response(&mut data).unwrap();
        assert_eq!(concat!(r#"<?xml version="1.0"?>"#,
                           r#"<methodResponse>"#,
                           r#"<params>"#,
                           r#"<param>"#,
                           r#"<value><array><data>"#,
                           r#"<value><i4>1</i4></value>"#,
                           r#"<value><i4>2</i4></value>"#,
                           r#"<value><i4>3</i4></value>"#,
                           r#"<value><i4>4</i4></value>"#,
                           r#"<value><i4>5</i4></value>"#,
                           r#"</data></array></value>"#,
                           r#"</param>"#,
                           r#"</params>"#,
                           r#"</methodResponse>"#),
                   std::str::from_utf8(&data).unwrap());
    }

    #[derive(Debug,PartialEq,RustcEncodable)]
    struct ExampleTuple(i32, f64, String, bool);

    #[test]
    fn writes_tuple() {
        let mut data = vec![];
        let mut encoder = Encoder::new();
        ExampleTuple(5, 0.5, String::from("abc"), false).encode(&mut encoder).unwrap();
        encoder.write_response(&mut data).unwrap();
        assert_eq!(concat!(r#"<?xml version="1.0"?>"#,
                           r#"<methodResponse>"#,
                           r#"<params>"#,
                           r#"<param>"#,
                           r#"<value><array><data>"#,
                           r#"<value><i4>5</i4></value>"#,
                           r#"<value><double>0.5</double></value>"#,
                           r#"<value><string>abc</string></value>"#,
                           r#"<value><boolean>0</boolean></value>"#,
                           r#"</data></array></value>"#,
                           r#"</param>"#,
                           r#"</params>"#,
                           r#"</methodResponse>"#),
                   std::str::from_utf8(&data).unwrap());
    }

    #[derive(Debug,PartialEq,RustcEncodable)]
    struct ExampleRequestStruct {
        a: i32,
        b: bool,
        c: ExampleRequestStructChild,
    }

    #[derive(Debug,PartialEq,RustcEncodable)]
    struct ExampleRequestStructChild {
        a: String,
        b: f64,
    }

    #[test]
    fn writes_struct() {
        let mut data = vec![];
        let mut encoder = Encoder::new();
        ExampleRequestStruct {
                a: 41,
                b: true,
                c: ExampleRequestStructChild {
                    a: String::from("Hello"),
                    b: 0.5,
                },
            }
            .encode(&mut encoder)
            .unwrap();
        encoder.write_response(&mut data).unwrap();
        assert_eq!(concat!(r#"<?xml version="1.0"?>"#,
                           r#"<methodResponse>"#,
                           r#"<params>"#,
                           r#"<param>"#,
                           r#"<value><array><data>"#,
                           r#"<value><i4>41</i4></value>"#,
                           r#"<value><boolean>1</boolean></value>"#,
                           r#"<value><array><data>"#,
                           r#"<value><string>Hello</string></value>"#,
                           r#"<value><double>0.5</double></value>"#,
                           r#"</data></array></value>"#,
                           r#"</data></array></value>"#,
                           r#"</param>"#,
                           r#"</params>"#,
                           r#"</methodResponse>"#),
                   std::str::from_utf8(&data).unwrap());
    }

    #[test]
    fn writes_multiple_parameters() {
        let mut data = vec![];
        let mut encoder = Encoder::new();
        ExampleTuple(5, 0.5, String::from("abc"), false).encode(&mut encoder).unwrap();
        27i32.encode(&mut encoder).unwrap();
        String::from("Hello").encode(&mut encoder).unwrap();
        ExampleRequestStruct {
                a: 41,
                b: true,
                c: ExampleRequestStructChild {
                    a: String::from("Hello"),
                    b: 0.5,
                },
            }
            .encode(&mut encoder)
            .unwrap();
        encoder.write_response(&mut data).unwrap();
        assert_eq!(concat!(r#"<?xml version="1.0"?>"#,
                           r#"<methodResponse>"#,
                           r#"<params>"#,
                           r#"<param>"#,
                           r#"<value><array><data>"#,
                           r#"<value><i4>5</i4></value>"#,
                           r#"<value><double>0.5</double></value>"#,
                           r#"<value><string>abc</string></value>"#,
                           r#"<value><boolean>0</boolean></value>"#,
                           r#"</data></array></value>"#,
                           r#"</param>"#,
                           r#"<param>"#,
                           r#"<value><i4>27</i4></value>"#,
                           r#"</param>"#,
                           r#"<param>"#,
                           r#"<value><string>Hello</string></value>"#,
                           r#"</param>"#,
                           r#"<param>"#,
                           r#"<value><array><data>"#,
                           r#"<value><i4>41</i4></value>"#,
                           r#"<value><boolean>1</boolean></value>"#,
                           r#"<value><array><data>"#,
                           r#"<value><string>Hello</string></value>"#,
                           r#"<value><double>0.5</double></value>"#,
                           r#"</data></array></value>"#,
                           r#"</data></array></value>"#,
                           r#"</param>"#,
                           r#"</params>"#,
                           r#"</methodResponse>"#),
                   std::str::from_utf8(&data).unwrap());
    }
}
