use crate::error::{Result, ResultExt};
use lazy_static::lazy_static;
use proc_macro2::{Literal, Span};
use quote::{quote, ToTokens};
use regex::Regex;
use std::collections::{BTreeSet, HashMap};
use syn::Ident;

#[derive(Clone)]
pub struct Msg {
    pub package: String,
    pub name: String,
    pub fields: Vec<FieldInfo>,
    pub source: String,
}

impl Msg {
    pub fn new(package: &str, name: &str, source: &str) -> Result<Msg> {
        let fields = match_lines(source)?;
        Ok(Msg {
            package: package.to_owned(),
            name: name.to_owned(),
            fields,
            source: source.trim().into(),
        })
    }

    pub fn name_ident(&self) -> Ident {
        Ident::new(&self.name, Span::call_site())
    }

    pub fn token_stream<T: ToTokens>(&self, crate_prefix: &T) -> impl ToTokens {
        let name = self.name_ident();
        let fields = self
            .fields
            .iter()
            .map(|v| v.field_token_stream(crate_prefix))
            .collect::<Vec<_>>();
        let field_defaults = self
            .fields
            .iter()
            .map(|v| v.field_default_token_stream(crate_prefix))
            .collect::<Vec<_>>();
        let const_fields = self
            .fields
            .iter()
            .map(|v| v.const_token_stream(crate_prefix))
            .collect::<Vec<_>>();
        quote! {
            #[allow(dead_code, non_camel_case_types, non_snake_case)]
            #[derive(Clone)]
            pub struct #name {
                #(#fields)*
            }
            impl #name {
                #(#const_fields)*
            }

            impl Default for #name {
                fn default() -> Self {
                    Self {
                        #(#field_defaults)*
                    }
                }
            }
        }
    }

    pub fn token_stream_encode<T: ToTokens>(&self, crate_prefix: &T) -> impl ToTokens {
        let fields = self
            .fields
            .iter()
            .map(|v| v.field_token_stream_encode(crate_prefix))
            .collect::<Vec<_>>();
        quote! {
            #(#fields)*
            Ok(())
        }
    }

    pub fn token_stream_decode<T: ToTokens>(&self, crate_prefix: &T) -> impl ToTokens {
        let fields = self
            .fields
            .iter()
            .map(|v| v.field_token_stream_decode(crate_prefix))
            .collect::<Vec<_>>();
        quote! {
            Ok(Self {
                #(#fields)*
            })
        }
    }

    pub fn get_type(&self) -> String {
        format!("{}/{}", self.package, self.name)
    }

    pub fn dependencies(&self) -> Vec<(String, String)> {
        self.fields
            .iter()
            .filter_map(|field| match field.datatype {
                DataType::LocalStruct(ref name) => Some((self.package.clone(), name.clone())),
                DataType::RemoteStruct(ref pkg, ref name) => Some((pkg.clone(), name.clone())),
                _ => None,
            })
            .collect()
    }

    #[cfg(test)]
    pub fn calculate_md5(
        &self,
        hashes: &HashMap<(String, String), String>,
    ) -> ::std::result::Result<String, ()> {
        use md5::{Digest, Md5};

        let mut hasher = Md5::new();
        hasher.input(&self.get_md5_representation(hashes)?);
        Ok(hex::encode(hasher.result().as_slice()))
    }

    pub fn get_md5_representation(
        &self,
        hashes: &HashMap<(String, String), String>,
    ) -> ::std::result::Result<String, ()> {
        let constants = self
            .fields
            .iter()
            .filter(|v| v.is_constant())
            .map(|v| v.md5_string(&self.package, hashes))
            .collect::<::std::result::Result<Vec<String>, ()>>()?;
        let fields = self
            .fields
            .iter()
            .filter(|v| !v.is_constant())
            .map(|v| v.md5_string(&self.package, hashes))
            .collect::<::std::result::Result<Vec<String>, ()>>()?;
        let representation = constants
            .into_iter()
            .chain(fields)
            .collect::<Vec<_>>()
            .join("\n");
        Ok(representation)
    }

    pub fn has_header(&self) -> bool {
        self.fields.iter().any(FieldInfo::is_header)
    }

    pub fn header_token_stream<T: ToTokens>(&self, crate_prefix: &T) -> impl ToTokens {
        if !self.has_header() {
            return quote! {};
        }
        quote! {
            fn set_header(
                &mut self,
                clock: &::std::sync::Arc<#crate_prefix Clock>,
                seq: &::std::sync::Arc<::std::sync::atomic::AtomicUsize>,
            ) {
                if self.header.seq == 0 {
                    self.header.seq =
                        seq.fetch_add(1, ::std::sync::atomic::Ordering::SeqCst) as u32;
                }
                if self.header.stamp.nanos() == 0 {
                    self.header.stamp = clock.now();
                }
            }
        }
    }
}

static IGNORE_WHITESPACE: &'static str = r"\s*";
static ANY_WHITESPACE: &'static str = r"\s+";
static FIELD_TYPE: &'static str = r"([a-zA-Z0-9_/]+)";
static FIELD_NAME: &'static str = r"([a-zA-Z][a-zA-Z0-9_]*)";
static EMPTY_BRACKETS: &'static str = r"\[\s*\]";
static NUMBER_BRACKETS: &'static str = r"\[\s*([0-9]+)\s*\]";

lazy_static! {
    static ref RESERVED_KEYWORDS: BTreeSet<String> = [
        "as", "break", "const", "continue", "crate", "else", "enum", "extern", "false", "fn",
        "for", "if", "impl", "in", "let", "loop", "match", "mod", "move", "mut", "pub", "ref",
        "return", "Self", "self", "static", "struct", "super", "trait", "true", "type", "unsafe",
        "use", "where", "while", "abstract", "alignof", "become", "box", "do", "final", "macro",
        "offsetof", "override", "priv", "proc", "pure", "sizeof", "typeof", "unsized", "virtual",
        "yield",
    ]
    .iter()
    .map(|&item| String::from(item))
    .collect();
}

fn match_field(data: &str) -> Option<FieldLine> {
    lazy_static! {
        static ref MATCHER: String = format!("^{}{}{}$", FIELD_TYPE, ANY_WHITESPACE, FIELD_NAME);
        static ref RE: Regex = Regex::new(&MATCHER).unwrap();
    }
    let captures = match RE.captures(data) {
        Some(v) => v,
        None => return None,
    };
    Some(FieldLine {
        field_type: captures.get(1).unwrap().as_str().into(),
        field_name: captures.get(2).unwrap().as_str().into(),
    })
}

fn match_vector_field(data: &str) -> Option<FieldLine> {
    lazy_static! {
        static ref MATCHER: String = format!(
            "^{}{}{}{}{}$",
            FIELD_TYPE, IGNORE_WHITESPACE, EMPTY_BRACKETS, ANY_WHITESPACE, FIELD_NAME
        );
        static ref RE: Regex = Regex::new(&MATCHER).unwrap();
    }
    let captures = match RE.captures(data) {
        Some(v) => v,
        None => return None,
    };
    Some(FieldLine {
        field_type: captures.get(1).unwrap().as_str().into(),
        field_name: captures.get(2).unwrap().as_str().into(),
    })
}

fn match_array_field(data: &str) -> Option<(FieldLine, usize)> {
    lazy_static! {
        static ref MATCHER: String = format!(
            "^{}{}{}{}{}$",
            FIELD_TYPE, IGNORE_WHITESPACE, NUMBER_BRACKETS, ANY_WHITESPACE, FIELD_NAME
        );
        static ref RE: Regex = Regex::new(&MATCHER).unwrap();
    }
    let captures = match RE.captures(data) {
        Some(v) => v,
        None => return None,
    };
    Some((
        FieldLine {
            field_type: captures.get(1).unwrap().as_str().into(),
            field_name: captures.get(3).unwrap().as_str().into(),
        },
        captures.get(2).unwrap().as_str().parse().unwrap(),
    ))
}

fn match_const_string(data: &str) -> Option<(FieldLine, String)> {
    lazy_static! {
        static ref MATCHER: String = format!(
            r"^(string){}{}{}={}(.*)$",
            ANY_WHITESPACE, FIELD_NAME, IGNORE_WHITESPACE, IGNORE_WHITESPACE
        );
        static ref RE: Regex = Regex::new(&MATCHER).unwrap();
    }
    let captures = match RE.captures(data) {
        Some(v) => v,
        None => return None,
    };
    Some((
        FieldLine {
            field_type: captures.get(1).unwrap().as_str().into(),
            field_name: captures.get(2).unwrap().as_str().into(),
        },
        captures.get(3).unwrap().as_str().into(),
    ))
}

fn match_const_numeric(data: &str) -> Option<(FieldLine, String)> {
    lazy_static! {
        static ref MATCHER: String = format!(
            r"^{}{}{}{}={}(-?[0-9]+)$",
            FIELD_TYPE, ANY_WHITESPACE, FIELD_NAME, IGNORE_WHITESPACE, IGNORE_WHITESPACE
        );
        static ref RE: Regex = Regex::new(&MATCHER).unwrap();
    }
    let captures = match RE.captures(data) {
        Some(v) => v,
        None => return None,
    };
    Some((
        FieldLine {
            field_type: captures.get(1).unwrap().as_str().into(),
            field_name: captures.get(2).unwrap().as_str().into(),
        },
        captures.get(3).unwrap().as_str().into(),
    ))
}

fn match_line(data: &str) -> Option<Result<FieldInfo>> {
    if let Some((info, data)) = match_const_string(data.trim()) {
        return Some(FieldInfo::new(
            &info.field_type,
            &info.field_name,
            FieldCase::Const(data),
        ));
    }
    let data = match strip_useless(data) {
        Ok(v) => v,
        Err(v) => return Some(Err(v)),
    };

    if data == "" {
        return None;
    }
    if let Some(info) = match_field(data) {
        return Some(FieldInfo::new(
            &info.field_type,
            &info.field_name,
            FieldCase::Unit,
        ));
    }
    if let Some(info) = match_vector_field(data) {
        return Some(FieldInfo::new(
            &info.field_type,
            &info.field_name,
            FieldCase::Vector,
        ));
    }
    if let Some((info, count)) = match_array_field(data) {
        return Some(FieldInfo::new(
            &info.field_type,
            &info.field_name,
            FieldCase::Array(count),
        ));
    }
    if let Some((info, data)) = match_const_numeric(data) {
        return Some(FieldInfo::new(
            &info.field_type,
            &info.field_name,
            FieldCase::Const(data),
        ));
    }
    Some(Err(format!("Unsupported content of line: {}", data).into()))
}

#[inline]
fn strip_useless(data: &str) -> Result<&str> {
    Ok(data
        .splitn(2, '#')
        .next()
        .ok_or_else(|| {
            format!(
                "Somehow splitting a line resulted in 0 parts?! Happened here: {}",
                data
            )
        })?
        .trim())
}

#[inline]
fn match_lines(data: &str) -> Result<Vec<FieldInfo>> {
    data.split('\n')
        .filter_map(match_line)
        .collect::<Result<_>>()
        .chain_err(|| "Failed to parse line in data string")
}

#[derive(Debug, PartialEq)]
struct FieldLine {
    field_type: String,
    field_name: String,
}

#[derive(Clone, Debug, PartialEq)]
pub enum FieldCase {
    Unit,
    Vector,
    Array(usize),
    Const(String),
}

#[derive(Clone, Debug, PartialEq)]
pub struct FieldInfo {
    pub datatype: DataType,
    pub name: String,
    pub case: FieldCase,
}

impl FieldInfo {
    #[allow(dead_code)]
    pub fn create_identifier(&self, span: Span) -> Ident {
        if RESERVED_KEYWORDS.contains(&self.name) {
            return Ident::new(&format!("_{}", self.name), span);
        }
        Ident::new(&self.name, span)
    }

    fn is_constant(&self) -> bool {
        match self.case {
            FieldCase::Const(..) => true,
            _ => false,
        }
    }

    pub fn field_token_stream<T: ToTokens>(&self, crate_prefix: &T) -> impl ToTokens {
        let datatype = self.datatype.token_stream(crate_prefix);
        let name = self.create_identifier(Span::call_site());
        match self.case {
            FieldCase::Unit => quote! { pub #name: #datatype, },
            FieldCase::Vector => quote! { pub #name: Vec<#datatype>, },
            FieldCase::Array(l) => quote! { pub #name: [#datatype; #l], },
            FieldCase::Const(_) => quote! {},
        }
    }

    pub fn field_default_token_stream<T: ToTokens>(&self, _crate_prefix: &T) -> impl ToTokens {
        let name = self.create_identifier(Span::call_site());
        match self.case {
            FieldCase::Unit | FieldCase::Vector => quote! { #name: Default::default(), },
            FieldCase::Array(l) => quote! { #name: [Default::default(); #l], },
            FieldCase::Const(_) => quote! {},
        }
    }

    pub fn field_token_stream_encode<T: ToTokens>(&self, crate_prefix: &T) -> impl ToTokens {
        let name = self.create_identifier(Span::call_site());
        match self.case {
            FieldCase::Unit => quote! { self.#name.encode(w.by_ref())?; },
            FieldCase::Vector => match self.datatype {
                DataType::String
                | DataType::Time
                | DataType::Duration
                | DataType::LocalStruct(_)
                | DataType::RemoteStruct(_, _) => {
                    quote! { #crate_prefix rosmsg::encode_variable_slice(&self.#name, w.by_ref())?; }
                }
                _ => {
                    quote! { #crate_prefix rosmsg::encode_variable_primitive_slice(&self.#name, w.by_ref())?; }
                }
            },
            FieldCase::Array(_l) => {
                quote! { #crate_prefix rosmsg::encode_fixed_slice(&self.#name, w.by_ref())?; }
            }
            FieldCase::Const(_) => quote! {},
        }
    }

    pub fn field_token_stream_decode<T: ToTokens>(&self, crate_prefix: &T) -> impl ToTokens {
        let name = self.create_identifier(Span::call_site());
        match self.case {
            FieldCase::Unit => quote! { #name: #crate_prefix rosmsg::RosMsg::decode(r.by_ref())?, },
            FieldCase::Vector => match self.datatype {
                DataType::String
                | DataType::Time
                | DataType::Duration
                | DataType::LocalStruct(_)
                | DataType::RemoteStruct(_, _) => {
                    quote! { #name: #crate_prefix rosmsg::decode_variable_vec(r.by_ref())?, }
                }
                _ => {
                    quote! { #name: #crate_prefix rosmsg::decode_variable_primitive_vec(r.by_ref())?, }
                }
            },
            FieldCase::Array(l) => {
                let lines =
                    (0..l).map(|_| quote! { #crate_prefix rosmsg::RosMsg::decode(r.by_ref())?, });
                quote! { #name: [#(#lines)*], }
            }
            FieldCase::Const(_) => quote! {},
        }
    }

    pub fn const_token_stream<T: ToTokens>(&self, crate_prefix: &T) -> impl ToTokens {
        let value = match self.case {
            FieldCase::Const(ref value) => value,
            _ => return quote! {},
        };
        let name = self.create_identifier(Span::call_site());
        let datatype = self.datatype.token_stream(crate_prefix);
        let insides = match self.datatype {
            DataType::Bool => {
                let bool_value = if value != "0" {
                    quote! { true }
                } else {
                    quote! { false }
                };
                quote! { #name; bool = #bool_value }
            }
            DataType::String => quote! { #name: &'static str = #value },
            DataType::Time
            | DataType::Duration
            | DataType::LocalStruct(..)
            | DataType::RemoteStruct(..) => return quote! {},
            DataType::I8(_) => {
                let numeric_value = Literal::i8_suffixed(value.parse().unwrap());
                quote! { #name: #datatype = #numeric_value as #datatype }
            }
            DataType::I16 => {
                let numeric_value = Literal::i16_suffixed(value.parse().unwrap());
                quote! { #name: #datatype = #numeric_value as #datatype }
            }
            DataType::I32 => {
                let numeric_value = Literal::i32_suffixed(value.parse().unwrap());
                quote! { #name: #datatype = #numeric_value as #datatype }
            }
            DataType::I64 => {
                let numeric_value = Literal::i64_suffixed(value.parse().unwrap());
                quote! { #name: #datatype = #numeric_value as #datatype }
            }
            DataType::U8(_) => {
                let numeric_value = Literal::u8_suffixed(value.parse().unwrap());
                quote! { #name: #datatype = #numeric_value as #datatype }
            }
            DataType::U16 => {
                let numeric_value = Literal::u16_suffixed(value.parse().unwrap());
                quote! { #name: #datatype = #numeric_value as #datatype }
            }
            DataType::U32 => {
                let numeric_value = Literal::u32_suffixed(value.parse().unwrap());
                quote! { #name: #datatype = #numeric_value as #datatype }
            }
            DataType::U64 => {
                let numeric_value = Literal::u64_suffixed(value.parse().unwrap());
                quote! { #name: #datatype = #numeric_value as #datatype }
            }
            DataType::F32 => {
                let numeric_value = Literal::f32_suffixed(value.parse().unwrap());
                quote! { #name: #datatype = #numeric_value as #datatype }
            }
            DataType::F64 => {
                let numeric_value = Literal::f64_suffixed(value.parse().unwrap());
                quote! { #name: #datatype = #numeric_value as #datatype }
            }
        };
        quote! {
            #[allow(dead_code,non_upper_case_globals)]
            pub const #insides;
        }
    }

    fn md5_string(
        &self,
        package: &str,
        hashes: &HashMap<(String, String), String>,
    ) -> ::std::result::Result<String, ()> {
        let datatype = self.datatype.md5_string(package, hashes)?;
        Ok(match (self.datatype.is_builtin(), &self.case) {
            (_, &FieldCase::Const(ref v)) => format!("{} {}={}", datatype, self.name, v),
            (false, _) | (_, &FieldCase::Unit) => format!("{} {}", datatype, self.name),
            (true, &FieldCase::Vector) => format!("{}[] {}", datatype, self.name),
            (true, &FieldCase::Array(l)) => format!("{}[{}] {}", datatype, l, self.name),
        })
    }

    fn is_header(&self) -> bool {
        self.case == FieldCase::Unit
            && self.name == "header"
            && self.datatype == DataType::RemoteStruct("std_msgs".into(), "Header".into())
    }

    fn new(datatype: &str, name: &str, case: FieldCase) -> Result<FieldInfo> {
        Ok(FieldInfo {
            datatype: parse_datatype(datatype)
                .ok_or_else(|| format!("Unsupported datatype: {}", datatype))?,
            name: name.to_owned(),
            case,
        })
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum DataType {
    Bool,
    I8(bool),
    I16,
    I32,
    I64,
    U8(bool),
    U16,
    U32,
    U64,
    F32,
    F64,
    String,
    Time,
    Duration,
    LocalStruct(String),
    RemoteStruct(String, String),
}

impl DataType {
    pub fn token_stream<T: ToTokens>(&self, crate_prefix: &T) -> impl ToTokens {
        match *self {
            DataType::Bool => quote! { bool },
            DataType::I8(_) => quote! { i8 },
            DataType::I16 => quote! { i16 },
            DataType::I32 => quote! { i32 },
            DataType::I64 => quote! { i64 },
            DataType::U8(_) => quote! { u8 },
            DataType::U16 => quote! { u16 },
            DataType::U32 => quote! { u32 },
            DataType::U64 => quote! { u64 },
            DataType::F32 => quote! { f32 },
            DataType::F64 => quote! { f64 },
            DataType::String => quote! { ::std::string::String },
            DataType::Time => quote! { #crate_prefix Time },
            DataType::Duration => quote! { #crate_prefix Duration },
            DataType::LocalStruct(ref name) => {
                let name = Ident::new(&name, Span::call_site());
                quote! { #name }
            }
            DataType::RemoteStruct(ref pkg, ref name) => {
                let name = Ident::new(&name, Span::call_site());
                let pkg = Ident::new(&pkg, Span::call_site());
                quote! { super::#pkg::#name }
            }
        }
    }

    fn is_builtin(&self) -> bool {
        match *self {
            DataType::Bool
            | DataType::I8(_)
            | DataType::I16
            | DataType::I32
            | DataType::I64
            | DataType::U8(_)
            | DataType::U16
            | DataType::U32
            | DataType::U64
            | DataType::F32
            | DataType::F64
            | DataType::String
            | DataType::Time
            | DataType::Duration => true,
            DataType::LocalStruct(_) | DataType::RemoteStruct(_, _) => false,
        }
    }

    fn md5_string(
        &self,
        package: &str,
        hashes: &HashMap<(String, String), String>,
    ) -> ::std::result::Result<String, ()> {
        Ok(match *self {
            DataType::Bool => "bool",
            DataType::I8(true) => "int8",
            DataType::I8(false) => "byte",
            DataType::I16 => "int16",
            DataType::I32 => "int32",
            DataType::I64 => "int64",
            DataType::U8(true) => "uint8",
            DataType::U8(false) => "char",
            DataType::U16 => "uint16",
            DataType::U32 => "uint32",
            DataType::U64 => "uint64",
            DataType::F32 => "float32",
            DataType::F64 => "float64",
            DataType::String => "string",
            DataType::Time => "time",
            DataType::Duration => "duration",
            DataType::LocalStruct(ref name) => hashes
                .get(&(package.to_owned(), name.clone()))
                .ok_or(())?
                .as_str(),
            DataType::RemoteStruct(ref pkg, ref name) => {
                hashes.get(&(pkg.clone(), name.clone())).ok_or(())?.as_str()
            }
        }
        .into())
    }
}

fn parse_datatype(datatype: &str) -> Option<DataType> {
    match datatype {
        "bool" => Some(DataType::Bool),
        "int8" => Some(DataType::I8(true)),
        "byte" => Some(DataType::I8(false)),
        "int16" => Some(DataType::I16),
        "int32" => Some(DataType::I32),
        "int64" => Some(DataType::I64),
        "uint8" => Some(DataType::U8(true)),
        "char" => Some(DataType::U8(false)),
        "uint16" => Some(DataType::U16),
        "uint32" => Some(DataType::U32),
        "uint64" => Some(DataType::U64),
        "float32" => Some(DataType::F32),
        "float64" => Some(DataType::F64),
        "string" => Some(DataType::String),
        "time" => Some(DataType::Time),
        "duration" => Some(DataType::Duration),
        "Header" => Some(DataType::RemoteStruct("std_msgs".into(), "Header".into())),
        _ => {
            let parts = datatype.split('/').collect::<Vec<_>>();
            if parts.iter().any(|v| v.is_empty()) {
                return None;
            }
            match parts.len() {
                2 => Some(DataType::RemoteStruct(
                    parts[0].to_owned(),
                    parts[1].to_owned(),
                )),
                1 => Some(DataType::LocalStruct(parts[0].to_owned())),
                _ => None,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn datatype_md5_string_correct() {
        let mut hashes = HashMap::new();
        hashes.insert(("p1".into(), "xx".into()), "ABCD".into());
        hashes.insert(("p2".into(), "xx".into()), "EFGH".into());
        assert_eq!(
            DataType::I64.md5_string("", &hashes).unwrap(),
            "int64".to_owned()
        );
        assert_eq!(
            DataType::F32.md5_string("", &hashes).unwrap(),
            "float32".to_owned()
        );
        assert_eq!(
            DataType::String.md5_string("", &hashes).unwrap(),
            "string".to_owned()
        );
        assert_eq!(
            DataType::LocalStruct("xx".into())
                .md5_string("p1", &hashes)
                .unwrap(),
            "ABCD".to_owned()
        );
        assert_eq!(
            DataType::LocalStruct("xx".into())
                .md5_string("p2", &hashes)
                .unwrap(),
            "EFGH".to_owned()
        );
        assert_eq!(
            DataType::RemoteStruct("p1".into(), "xx".into())
                .md5_string("p2", &hashes)
                .unwrap(),
            "ABCD".to_owned()
        );
    }

    #[test]
    fn fieldinfo_md5_string_correct() {
        let mut hashes = HashMap::new();
        hashes.insert(("p1".into(), "xx".into()), "ABCD".into());
        hashes.insert(("p2".into(), "xx".into()), "EFGH".into());
        assert_eq!(
            FieldInfo::new("int64", "abc", FieldCase::Unit)
                .unwrap()
                .md5_string("", &hashes)
                .unwrap(),
            "int64 abc".to_owned()
        );
        assert_eq!(
            FieldInfo::new("float32", "abc", FieldCase::Array(3))
                .unwrap()
                .md5_string("", &hashes)
                .unwrap(),
            "float32[3] abc".to_owned()
        );
        assert_eq!(
            FieldInfo::new("int32", "abc", FieldCase::Vector)
                .unwrap()
                .md5_string("", &hashes)
                .unwrap(),
            "int32[] abc".to_owned()
        );
        assert_eq!(
            FieldInfo::new("string", "abc", FieldCase::Const("something".into()))
                .unwrap()
                .md5_string("", &hashes)
                .unwrap(),
            "string abc=something".to_owned()
        );
        assert_eq!(
            FieldInfo::new("xx", "abc", FieldCase::Vector)
                .unwrap()
                .md5_string("p1", &hashes)
                .unwrap(),
            "ABCD abc".to_owned()
        );
        assert_eq!(
            FieldInfo::new("xx", "abc", FieldCase::Array(3))
                .unwrap()
                .md5_string("p1", &hashes)
                .unwrap(),
            "ABCD abc".to_owned()
        );
        assert_eq!(
            FieldInfo::new("p2/xx", "abc", FieldCase::Unit)
                .unwrap()
                .md5_string("p1", &hashes)
                .unwrap(),
            "EFGH abc".to_owned()
        );
    }

    #[test]
    fn message_md5_string_correct() {
        assert_eq!(
            Msg::new("std_msgs", "String", "string data")
                .unwrap()
                .calculate_md5(&HashMap::new())
                .unwrap(),
            "992ce8a1687cec8c8bd883ec73ca41d1".to_owned()
        );
        assert_eq!(
            Msg::new(
                "geometry_msgs",
                "Point",
                include_str!("msg_examples/geometry_msgs/msg/Point.msg"),
            )
            .unwrap()
            .calculate_md5(&HashMap::new())
            .unwrap(),
            "4a842b65f413084dc2b10fb484ea7f17".to_owned()
        );
        assert_eq!(
            Msg::new(
                "geometry_msgs",
                "Quaternion",
                include_str!("msg_examples/geometry_msgs/msg/Quaternion.msg"),
            )
            .unwrap()
            .calculate_md5(&HashMap::new())
            .unwrap(),
            "a779879fadf0160734f906b8c19c7004".to_owned()
        );
        let mut hashes = HashMap::new();
        hashes.insert(
            ("geometry_msgs".into(), "Point".into()),
            "4a842b65f413084dc2b10fb484ea7f17".into(),
        );
        hashes.insert(
            ("geometry_msgs".into(), "Quaternion".into()),
            "a779879fadf0160734f906b8c19c7004".into(),
        );
        assert_eq!(
            Msg::new(
                "geometry_msgs",
                "Pose",
                include_str!("msg_examples/geometry_msgs/msg/Pose.msg"),
            )
            .unwrap()
            .calculate_md5(&hashes)
            .unwrap(),
            "e45d45a5a1ce597b249e23fb30fc871f".to_owned()
        );
        let mut hashes = HashMap::new();
        hashes.insert(
            ("geometry_msgs".into(), "Point".into()),
            "4a842b65f413084dc2b10fb484ea7f17".into(),
        );
        hashes.insert(
            ("std_msgs".into(), "ColorRGBA".into()),
            "a29a96539573343b1310c73607334b00".into(),
        );
        hashes.insert(
            ("std_msgs".into(), "Header".into()),
            "2176decaecbce78abc3b96ef049fabed".into(),
        );
        assert_eq!(
            Msg::new(
                "visualization_msgs",
                "ImageMarker",
                include_str!("msg_examples/visualization_msgs/msg/ImageMarker.msg"),
            )
            .unwrap()
            .calculate_md5(&hashes)
            .unwrap(),
            "1de93c67ec8858b831025a08fbf1b35c".to_owned()
        );
    }

    #[test]
    fn match_field_matches_legal_field() {
        assert_eq!(
            FieldLine {
                field_type: "geom_msgs/Twist".into(),
                field_name: "myname".into(),
            },
            match_field("geom_msgs/Twist   myname").unwrap()
        );
    }

    #[test]
    fn match_vector_field_matches_legal_field() {
        assert_eq!(
            FieldLine {
                field_type: "geom_msgs/Twist".into(),
                field_name: "myname".into(),
            },
            match_vector_field("geom_msgs/Twist [  ]   myname").unwrap()
        );
    }

    #[test]
    fn match_array_field_matches_legal_field() {
        assert_eq!(
            (
                FieldLine {
                    field_type: "geom_msgs/Twist".into(),
                    field_name: "myname".into(),
                },
                127,
            ),
            match_array_field("geom_msgs/Twist   [   127 ]   myname").unwrap()
        );
    }

    #[test]
    fn match_const_string_matches_legal_field() {
        assert_eq!(
            (
                FieldLine {
                    field_type: "string".into(),
                    field_name: "myname".into(),
                },
                "this is # data".into(),
            ),
            match_const_string("string   myname  =  this is # data").unwrap()
        );
    }

    #[test]
    fn match_const_numeric_matches_legal_field() {
        assert_eq!(
            (
                FieldLine {
                    field_type: "mytype".into(),
                    field_name: "myname".into(),
                },
                "-444".into(),
            ),
            match_const_numeric("mytype   myname  =  -444").unwrap()
        );
    }

    #[test]
    fn match_line_works_on_legal_data() {
        assert!(match_line("#just a comment").is_none());
        assert!(match_line("#  YOLO !   ").is_none());
        assert!(match_line("      ").is_none());

        assert_eq!(
            FieldInfo {
                datatype: DataType::RemoteStruct("geom_msgs".into(), "Twist".into()),
                name: "myname".into(),
                case: FieldCase::Unit,
            },
            match_line("  geom_msgs/Twist   myname    # this clearly should succeed",)
                .unwrap()
                .unwrap()
        );

        assert_eq!(
            FieldInfo {
                datatype: DataType::RemoteStruct("geom_msgs".into(), "Twist".into()),
                name: "myname".into(),
                case: FieldCase::Vector,
            },
            match_line("  geom_msgs/Twist [  ]   myname  # ...")
                .unwrap()
                .unwrap()
        );

        assert_eq!(
            FieldInfo {
                datatype: DataType::U8(false),
                name: "myname".into(),
                case: FieldCase::Array(127),
            },
            match_line("  char   [   127 ]   myname# comment")
                .unwrap()
                .unwrap()
        );
        assert_eq!(
            FieldInfo {
                datatype: DataType::String,
                name: "myname".into(),
                case: FieldCase::Const("this is # data".into()),
            },
            match_line("  string  myname =   this is # data  ")
                .unwrap()
                .unwrap()
        );
        assert_eq!(
            FieldInfo {
                datatype: DataType::RemoteStruct("geom_msgs".into(), "Twist".into()),
                name: "myname".into(),
                case: FieldCase::Const("-444".into()),
            },
            match_line("  geom_msgs/Twist  myname =   -444 # data  ")
                .unwrap()
                .unwrap()
        );
    }

    #[test]
    fn match_lines_parses_real_messages() {
        let data = match_lines(include_str!(
            "msg_examples/geometry_msgs/msg/TwistWithCovariance.\
             msg"
        ))
        .unwrap();
        assert_eq!(
            vec![
                FieldInfo {
                    datatype: DataType::LocalStruct("Twist".into()),
                    name: "twist".into(),
                    case: FieldCase::Unit,
                },
                FieldInfo {
                    datatype: DataType::F64,
                    name: "covariance".into(),
                    case: FieldCase::Array(36),
                },
            ],
            data
        );

        let data = match_lines(include_str!(
            "msg_examples/geometry_msgs/msg/PoseStamped.msg"
        ))
        .unwrap();
        assert_eq!(
            vec![
                FieldInfo {
                    datatype: DataType::RemoteStruct("std_msgs".into(), "Header".into()),
                    name: "header".into(),
                    case: FieldCase::Unit,
                },
                FieldInfo {
                    datatype: DataType::LocalStruct("Pose".into()),
                    name: "pose".into(),
                    case: FieldCase::Unit,
                },
            ],
            data
        );
    }

    fn get_dependency_set(message: &Msg) -> HashSet<(String, String)> {
        message.dependencies().into_iter().collect()
    }

    #[test]
    fn msg_constructor_parses_real_message() {
        let data = Msg::new(
            "geometry_msgs",
            "TwistWithCovariance",
            include_str!("msg_examples/geometry_msgs/msg/TwistWithCovariance.msg"),
        )
        .unwrap();
        assert_eq!(data.package, "geometry_msgs");
        assert_eq!(data.name, "TwistWithCovariance");
        assert_eq!(
            data.fields,
            vec![
                FieldInfo {
                    datatype: DataType::LocalStruct("Twist".into()),
                    name: "twist".into(),
                    case: FieldCase::Unit,
                },
                FieldInfo {
                    datatype: DataType::F64,
                    name: "covariance".into(),
                    case: FieldCase::Array(36),
                },
            ]
        );
        let dependencies = get_dependency_set(&data);
        assert_eq!(dependencies.len(), 1);
        assert!(dependencies.contains(&("geometry_msgs".into(), "Twist".into()),));

        let data = Msg::new(
            "geometry_msgs",
            "PoseStamped",
            include_str!("msg_examples/geometry_msgs/msg/PoseStamped.msg"),
        )
        .unwrap();
        assert_eq!(data.package, "geometry_msgs");
        assert_eq!(data.name, "PoseStamped");
        assert_eq!(
            data.fields,
            vec![
                FieldInfo {
                    datatype: DataType::RemoteStruct("std_msgs".into(), "Header".into()),
                    name: "header".into(),
                    case: FieldCase::Unit,
                },
                FieldInfo {
                    datatype: DataType::LocalStruct("Pose".into()),
                    name: "pose".into(),
                    case: FieldCase::Unit,
                },
            ]
        );
        let dependencies = get_dependency_set(&data);
        assert_eq!(dependencies.len(), 2);
        assert!(dependencies.contains(&("geometry_msgs".into(), "Pose".into()),));
        assert!(dependencies.contains(&("std_msgs".into(), "Header".into())));

        let data = Msg::new(
            "sensor_msgs",
            "Imu",
            include_str!("msg_examples/sensor_msgs/msg/Imu.msg"),
        )
        .unwrap();
        assert_eq!(data.package, "sensor_msgs");
        assert_eq!(data.name, "Imu");
        assert_eq!(
            data.fields,
            vec![
                FieldInfo {
                    datatype: DataType::RemoteStruct("std_msgs".into(), "Header".into()),
                    name: "header".into(),
                    case: FieldCase::Unit,
                },
                FieldInfo {
                    datatype: DataType::RemoteStruct("geometry_msgs".into(), "Quaternion".into()),
                    name: "orientation".into(),
                    case: FieldCase::Unit,
                },
                FieldInfo {
                    datatype: DataType::F64,
                    name: "orientation_covariance".into(),
                    case: FieldCase::Array(9),
                },
                FieldInfo {
                    datatype: DataType::RemoteStruct("geometry_msgs".into(), "Vector3".into()),
                    name: "angular_velocity".into(),
                    case: FieldCase::Unit,
                },
                FieldInfo {
                    datatype: DataType::F64,
                    name: "angular_velocity_covariance".into(),
                    case: FieldCase::Array(9),
                },
                FieldInfo {
                    datatype: DataType::RemoteStruct("geometry_msgs".into(), "Vector3".into()),
                    name: "linear_acceleration".into(),
                    case: FieldCase::Unit,
                },
                FieldInfo {
                    datatype: DataType::F64,
                    name: "linear_acceleration_covariance".into(),
                    case: FieldCase::Array(9),
                },
            ]
        );
        let dependencies = get_dependency_set(&data);
        assert_eq!(dependencies.len(), 3);
        assert!(dependencies.contains(&("geometry_msgs".into(), "Vector3".into()),));
        assert!(dependencies.contains(&("geometry_msgs".into(), "Quaternion".into()),));
        assert!(dependencies.contains(&("std_msgs".into(), "Header".into())));
    }
}
