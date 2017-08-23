use rustc_serialize;
use std;
use super::value::{self, XmlRpcValue};
use super::error::{ErrorKind, Result, ResultExt};

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

    fn form_subtree(
        mut data: &mut std::vec::IntoIter<(XmlRpcValue, usize)>,
    ) -> Option<XmlRpcValue> {
        match data.next() {
            Some((node, count)) => {
                Some(match node {
                    XmlRpcValue::Array(_) => {
                        XmlRpcValue::Array(
                            (0..count)
                                .into_iter()
                                .filter_map(|_| Encoder::form_subtree(&mut data))
                                .collect(),
                        )
                    }
                    _ => node,
                })
            }
            None => None,
        }
    }

    pub fn write_request<T: std::io::Write>(
        self,
        method: &str,
        mut body: &mut T,
    ) -> std::io::Result<()> {
        write!(
            &mut body,
            "{}",
            value::XmlRpcRequest {
                method: String::from(method),
                parameters: self.form_tree(),
            }
        )
    }

    pub fn write_response<T: std::io::Write>(self, mut body: &mut T) -> std::io::Result<()> {
        write!(
            &mut body,
            "{}",
            value::XmlRpcResponse { parameters: self.form_tree() }
        )
    }
}

type EncoderResult = Result<()>;

impl rustc_serialize::Encoder for Encoder {
    type Error = super::error::Error;

    fn emit_nil(&mut self) -> EncoderResult {
        bail!(ErrorKind::UnsupportedDataType("nil".into()))
    }

    fn emit_usize(&mut self, _: usize) -> EncoderResult {
        bail!(ErrorKind::UnsupportedDataType("usize".into()))
    }

    fn emit_u64(&mut self, _: u64) -> EncoderResult {
        bail!(ErrorKind::UnsupportedDataType("u64".into()))
    }

    fn emit_u32(&mut self, _: u32) -> EncoderResult {
        bail!(ErrorKind::UnsupportedDataType("u32".into()))
    }

    fn emit_u16(&mut self, _: u16) -> EncoderResult {
        bail!(ErrorKind::UnsupportedDataType("u16".into()))
    }

    fn emit_u8(&mut self, _: u8) -> EncoderResult {
        bail!(ErrorKind::UnsupportedDataType("u8".into()))
    }

    fn emit_isize(&mut self, _: isize) -> EncoderResult {
        bail!(ErrorKind::UnsupportedDataType("isize".into()))
    }

    fn emit_i64(&mut self, _: i64) -> EncoderResult {
        bail!(ErrorKind::UnsupportedDataType("i64".into()))
    }

    fn emit_i32(&mut self, v: i32) -> EncoderResult {
        Ok(self.data.push((XmlRpcValue::Int(v), 0)))
    }

    fn emit_i16(&mut self, _: i16) -> EncoderResult {
        bail!(ErrorKind::UnsupportedDataType("i16".into()))
    }

    fn emit_i8(&mut self, _: i8) -> EncoderResult {
        bail!(ErrorKind::UnsupportedDataType("i8".into()))
    }

    fn emit_bool(&mut self, v: bool) -> EncoderResult {
        Ok(self.data.push((XmlRpcValue::Bool(v), 0)))
    }

    fn emit_f64(&mut self, v: f64) -> EncoderResult {
        Ok(self.data.push((XmlRpcValue::Double(v), 0)))
    }

    fn emit_f32(&mut self, _: f32) -> EncoderResult {
        bail!(ErrorKind::UnsupportedDataType("f32".into()))
    }

    fn emit_char(&mut self, _: char) -> EncoderResult {
        bail!(ErrorKind::UnsupportedDataType("char".into()))
    }

    fn emit_str(&mut self, v: &str) -> EncoderResult {
        Ok(self.data.push((XmlRpcValue::String(String::from(v)), 0)))
    }

    fn emit_enum<F>(&mut self, _: &str, _: F) -> EncoderResult
    where
        F: FnOnce(&mut Self) -> EncoderResult,
    {
        bail!(ErrorKind::UnsupportedDataType("enum".into()))
    }

    fn emit_enum_variant<F>(&mut self, _: &str, _: usize, _: usize, _: F) -> EncoderResult
    where
        F: FnOnce(&mut Self) -> EncoderResult,
    {
        bail!(ErrorKind::UnsupportedDataType("enum variant".into()))
    }

    fn emit_enum_variant_arg<F>(&mut self, _: usize, _: F) -> EncoderResult
    where
        F: FnOnce(&mut Self) -> EncoderResult,
    {
        bail!(ErrorKind::UnsupportedDataType(
            "enum variant argument".into(),
        ))
    }

    fn emit_enum_struct_variant<F>(&mut self, _: &str, _: usize, _: usize, _: F) -> EncoderResult
    where
        F: FnOnce(&mut Self) -> EncoderResult,
    {
        bail!(ErrorKind::UnsupportedDataType("enum struct variant".into()))
    }

    fn emit_enum_struct_variant_field<F>(&mut self, _: &str, _: usize, _: F) -> EncoderResult
    where
        F: FnOnce(&mut Self) -> EncoderResult,
    {
        bail!(ErrorKind::UnsupportedDataType(
            "enum struct variant field".into(),
        ))
    }

    fn emit_struct<F>(&mut self, name: &str, l: usize, f: F) -> EncoderResult
    where
        F: FnOnce(&mut Self) -> EncoderResult,
    {
        self.data.push((XmlRpcValue::Array(vec![]), l));
        f(self).chain_err(|| {
            ErrorKind::UnsupportedDataType(format!("struct {}", name))
        })
    }

    fn emit_struct_field<F>(&mut self, name: &str, _: usize, f: F) -> EncoderResult
    where
        F: FnOnce(&mut Self) -> EncoderResult,
    {
        f(self).chain_err(|| ErrorKind::UnsupportedDataType(format!("field {}", name)))
    }

    fn emit_tuple<F>(&mut self, l: usize, f: F) -> EncoderResult
    where
        F: FnOnce(&mut Self) -> EncoderResult,
    {
        self.data.push((XmlRpcValue::Array(vec![]), l));
        f(self).chain_err(|| ErrorKind::UnsupportedDataType("tuple".into()))
    }

    fn emit_tuple_arg<F>(&mut self, n: usize, f: F) -> EncoderResult
    where
        F: FnOnce(&mut Self) -> EncoderResult,
    {
        f(self).chain_err(|| {
            ErrorKind::UnsupportedDataType(format!("field number {}", n))
        })
    }

    fn emit_tuple_struct<F>(&mut self, name: &str, l: usize, f: F) -> EncoderResult
    where
        F: FnOnce(&mut Self) -> EncoderResult,
    {
        self.data.push((XmlRpcValue::Array(vec![]), l));
        f(self).chain_err(|| {
            ErrorKind::UnsupportedDataType(format!("tuple struct {}", name))
        })
    }

    fn emit_tuple_struct_arg<F>(&mut self, n: usize, f: F) -> EncoderResult
    where
        F: FnOnce(&mut Self) -> EncoderResult,
    {
        f(self).chain_err(|| {
            ErrorKind::UnsupportedDataType(format!("field number {}", n))
        })
    }

    fn emit_option<F>(&mut self, _: F) -> EncoderResult
    where
        F: FnOnce(&mut Self) -> EncoderResult,
    {
        bail!(ErrorKind::UnsupportedDataType("option".into()))
    }

    fn emit_option_none(&mut self) -> EncoderResult {
        bail!(ErrorKind::UnsupportedDataType("none".into()))
    }

    fn emit_option_some<F>(&mut self, _: F) -> EncoderResult
    where
        F: FnOnce(&mut Self) -> EncoderResult,
    {
        bail!(ErrorKind::UnsupportedDataType("some".into()))
    }

    fn emit_seq<F>(&mut self, l: usize, f: F) -> EncoderResult
    where
        F: FnOnce(&mut Self) -> EncoderResult,
    {
        self.data.push((XmlRpcValue::Array(vec![]), l));
        f(self).chain_err(|| ErrorKind::UnsupportedDataType("array".into()))
    }

    fn emit_seq_elt<F>(&mut self, n: usize, f: F) -> EncoderResult
    where
        F: FnOnce(&mut Self) -> EncoderResult,
    {
        f(self).chain_err(|| {
            ErrorKind::UnsupportedDataType(format!("element number {}", n))
        })
    }

    fn emit_map<F>(&mut self, _: usize, _: F) -> EncoderResult
    where
        F: FnOnce(&mut Self) -> EncoderResult,
    {
        bail!(ErrorKind::UnsupportedDataType("map".into()))
    }

    fn emit_map_elt_key<F>(&mut self, _: usize, _: F) -> EncoderResult
    where
        F: FnOnce(&mut Self) -> EncoderResult,
    {
        bail!(ErrorKind::UnsupportedDataType("map element key".into()))
    }

    fn emit_map_elt_val<F>(&mut self, _: usize, _: F) -> EncoderResult
    where
        F: FnOnce(&mut Self) -> EncoderResult,
    {
        bail!(ErrorKind::UnsupportedDataType("map element value".into()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rustc_serialize::Encodable;
    use std;

    static FAILED_TO_ENCODE: &'static str = "Failed to encode";

    #[test]
    fn writes_response() {
        let mut data = vec![];
        let mut encoder = Encoder::new();
        String::from("Hello").encode(&mut encoder).expect(
            FAILED_TO_ENCODE,
        );
        encoder.write_response(&mut data).expect(FAILED_TO_ENCODE);
        assert_eq!(
            concat!(
                r#"<?xml version="1.0"?>"#,
                r#"<methodResponse>"#,
                r#"<params>"#,
                r#"<param>"#,
                r#"<value><string>Hello</string></value>"#,
                r#"</param>"#,
                r#"</params>"#,
                r#"</methodResponse>"#
            ),
            std::str::from_utf8(&data).expect(FAILED_TO_ENCODE)
        );
    }

    #[test]
    fn writes_request() {
        let mut data = vec![];
        let mut encoder = Encoder::new();
        String::from("Hello").encode(&mut encoder).expect(
            FAILED_TO_ENCODE,
        );
        encoder.write_request("something", &mut data).expect(
            FAILED_TO_ENCODE,
        );
        assert_eq!(
            concat!(
                r#"<?xml version="1.0"?>"#,
                r#"<methodCall>"#,
                r#"<methodName>"#,
                r#"something"#,
                r#"</methodName>"#,
                r#"<params>"#,
                r#"<param>"#,
                r#"<value><string>Hello</string></value>"#,
                r#"</param>"#,
                r#"</params>"#,
                r#"</methodCall>"#
            ),
            std::str::from_utf8(&data).expect(FAILED_TO_ENCODE)
        );
    }

    #[test]
    fn writes_string() {
        let mut data = vec![];
        let mut encoder = Encoder::new();
        String::from("Hello").encode(&mut encoder).expect(
            FAILED_TO_ENCODE,
        );
        String::from("There").encode(&mut encoder).expect(
            FAILED_TO_ENCODE,
        );
        String::from("Friend").encode(&mut encoder).expect(
            FAILED_TO_ENCODE,
        );
        encoder.write_response(&mut data).expect(FAILED_TO_ENCODE);
        assert_eq!(
            concat!(
                r#"<?xml version="1.0"?>"#,
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
                r#"</methodResponse>"#
            ),
            std::str::from_utf8(&data).expect(FAILED_TO_ENCODE)
        );
    }

    #[test]
    fn writes_int() {
        let mut data = vec![];
        let mut encoder = Encoder::new();
        43i32.encode(&mut encoder).expect(FAILED_TO_ENCODE);
        27i32.encode(&mut encoder).expect(FAILED_TO_ENCODE);
        12i32.encode(&mut encoder).expect(FAILED_TO_ENCODE);
        encoder.write_response(&mut data).expect(FAILED_TO_ENCODE);
        assert_eq!(
            concat!(
                r#"<?xml version="1.0"?>"#,
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
                r#"</methodResponse>"#
            ),
            std::str::from_utf8(&data).expect(FAILED_TO_ENCODE)
        );
    }

    #[test]
    fn writes_float() {
        let mut data = vec![];
        let mut encoder = Encoder::new();
        33.5f64.encode(&mut encoder).expect(FAILED_TO_ENCODE);
        11.25f64.encode(&mut encoder).expect(FAILED_TO_ENCODE);
        77.125f64.encode(&mut encoder).expect(FAILED_TO_ENCODE);
        encoder.write_response(&mut data).expect(FAILED_TO_ENCODE);
        assert_eq!(
            concat!(
                r#"<?xml version="1.0"?>"#,
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
                r#"</methodResponse>"#
            ),
            std::str::from_utf8(&data).expect(FAILED_TO_ENCODE)
        );
    }

    #[test]
    fn writes_bool() {
        let mut data = vec![];
        let mut encoder = Encoder::new();
        true.encode(&mut encoder).expect(FAILED_TO_ENCODE);
        false.encode(&mut encoder).expect(FAILED_TO_ENCODE);
        encoder.write_response(&mut data).expect(FAILED_TO_ENCODE);
        assert_eq!(
            concat!(
                r#"<?xml version="1.0"?>"#,
                r#"<methodResponse>"#,
                r#"<params>"#,
                r#"<param>"#,
                r#"<value><boolean>1</boolean></value>"#,
                r#"</param>"#,
                r#"<param>"#,
                r#"<value><boolean>0</boolean></value>"#,
                r#"</param>"#,
                r#"</params>"#,
                r#"</methodResponse>"#
            ),
            std::str::from_utf8(&data).expect(FAILED_TO_ENCODE)
        );
    }

    #[test]
    fn writes_array() {
        let mut data = vec![];
        let mut encoder = Encoder::new();
        vec![1i32, 2, 3, 4, 5].encode(&mut encoder).expect(
            FAILED_TO_ENCODE,
        );
        encoder.write_response(&mut data).expect(FAILED_TO_ENCODE);
        assert_eq!(
            concat!(
                r#"<?xml version="1.0"?>"#,
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
                r#"</methodResponse>"#
            ),
            std::str::from_utf8(&data).expect(FAILED_TO_ENCODE)
        );
    }

    #[derive(Debug, PartialEq, RustcEncodable)]
    struct ExampleTuple(i32, f64, String, bool);

    #[test]
    fn writes_tuple() {
        let mut data = vec![];
        let mut encoder = Encoder::new();
        ExampleTuple(5, 0.5, String::from("abc"), false)
            .encode(&mut encoder)
            .expect(FAILED_TO_ENCODE);
        encoder.write_response(&mut data).expect(FAILED_TO_ENCODE);
        assert_eq!(
            concat!(
                r#"<?xml version="1.0"?>"#,
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
                r#"</methodResponse>"#
            ),
            std::str::from_utf8(&data).expect(FAILED_TO_ENCODE)
        );
    }

    #[derive(Debug, PartialEq, RustcEncodable)]
    struct ExampleRequestStruct {
        a: i32,
        b: bool,
        c: ExampleRequestStructChild,
    }

    #[derive(Debug, PartialEq, RustcEncodable)]
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
        }.encode(&mut encoder)
            .expect(FAILED_TO_ENCODE);
        encoder.write_response(&mut data).expect(FAILED_TO_ENCODE);
        assert_eq!(
            concat!(
                r#"<?xml version="1.0"?>"#,
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
                r#"</methodResponse>"#
            ),
            std::str::from_utf8(&data).expect(FAILED_TO_ENCODE)
        );
    }

    #[test]
    fn writes_multiple_parameters() {
        let mut data = vec![];
        let mut encoder = Encoder::new();
        ExampleTuple(5, 0.5, String::from("abc"), false)
            .encode(&mut encoder)
            .expect(FAILED_TO_ENCODE);
        27i32.encode(&mut encoder).expect(FAILED_TO_ENCODE);
        String::from("Hello").encode(&mut encoder).expect(
            FAILED_TO_ENCODE,
        );
        ExampleRequestStruct {
            a: 41,
            b: true,
            c: ExampleRequestStructChild {
                a: String::from("Hello"),
                b: 0.5,
            },
        }.encode(&mut encoder)
            .expect(FAILED_TO_ENCODE);
        encoder.write_response(&mut data).expect(FAILED_TO_ENCODE);
        assert_eq!(
            concat!(
                r#"<?xml version="1.0"?>"#,
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
                r#"</methodResponse>"#
            ),
            std::str::from_utf8(&data).expect(FAILED_TO_ENCODE)
        );
    }
}
