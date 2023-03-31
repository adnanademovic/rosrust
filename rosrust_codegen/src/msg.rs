use crate::error::{Result, ResultExt};
use lazy_static::lazy_static;
use proc_macro2::{Literal, Span};
use quote::{quote, ToTokens};
use ros_message::{DataType, FieldCase, FieldInfo, MessagePath};
use std::collections::{BTreeSet, HashMap};
use syn::Ident;

#[derive(Clone, Debug)]
pub struct Msg(pub ros_message::Msg);

#[derive(Clone, Debug)]
pub struct Srv {
    pub path: MessagePath,
    pub source: String,
}

impl Msg {
    pub fn new(path: MessagePath, source: &str) -> Result<Msg> {
        ros_message::Msg::new(path, source)
            .map(Self)
            .chain_err(|| "Failed to parse message")
    }

    pub fn name_ident(&self) -> Ident {
        Ident::new(self.0.path().name(), Span::call_site())
    }

    pub fn token_stream<T: ToTokens>(&self, crate_prefix: &T) -> impl ToTokens {
        let name = self.name_ident();
        let fields = self
            .0
            .fields()
            .iter()
            .map(|v| field_info_field_token_stream(v, crate_prefix))
            .collect::<Vec<_>>();
        let field_defaults = self
            .0
            .fields()
            .iter()
            .map(|v| field_info_field_default_token_stream(v, crate_prefix))
            .collect::<Vec<_>>();
        let fields_into_values = self
            .0
            .fields()
            .iter()
            .map(|v| field_info_field_into_value_token_stream(v, crate_prefix))
            .collect::<Vec<_>>();
        let fields_from_values = self
            .0
            .fields()
            .iter()
            .map(|v| field_info_field_from_value_token_stream(v, crate_prefix))
            .collect::<Vec<_>>();
        let fields_for_eq_and_debug = self
            .0
            .fields()
            .iter()
            .filter_map(field_info_field_name_eq_and_debug_token_stream)
            .collect::<Vec<_>>();
        let fields_partialeq = fields_for_eq_and_debug
            .iter()
            .map(|(_, peq, _)| quote! { self.#peq == other.#peq })
            .collect::<Vec<_>>();
        let fields_debug = fields_for_eq_and_debug
            .iter()
            .map(|(name, _, dbg)| quote! { .field(stringify!(#name), #dbg) })
            .collect::<Vec<_>>();
        let const_fields = self
            .0
            .fields()
            .iter()
            .map(|v| field_info_const_token_stream(v, crate_prefix))
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

            impl std::convert::From<#name> for #crate_prefix MsgValue {
                fn from(src: #name) -> Self {
                    #crate_prefix MsgValue::Message(src.into())
                }
            }

            impl std::convert::From<#name> for #crate_prefix MsgMessage {
                fn from(src: #name) -> Self {
                    let mut output = Self::new();
                    #(#fields_into_values)*
                    output
                }
            }

            impl std::convert::TryFrom<#crate_prefix MsgValue> for #name {
                type Error = ();

                fn try_from(src: #crate_prefix MsgValue) -> Result<Self, ()> {
                    use std::convert::TryInto;
                    let message: #crate_prefix MsgMessage = src.try_into()?;
                    message.try_into()
                }
            }

            impl std::convert::TryFrom<#crate_prefix MsgMessage> for #name {
                type Error = ();

                fn try_from(mut src: #crate_prefix MsgMessage) -> Result<Self, ()> {
                    use std::convert::TryInto;
                    Ok(Self {
                        #(#fields_from_values)*
                    })
                }
            }

            impl std::cmp::PartialEq<Self> for #name {
                fn eq(&self, other: &Self) -> bool {
                    true #(&& #fields_partialeq)*
                }
            }

            impl std::fmt::Debug for #name {
                fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                    f.debug_struct(stringify!(#name))
                        #(#fields_debug)*
                        .finish()
                }
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
            .0
            .fields()
            .iter()
            .map(|v| field_info_field_token_stream_encode(v, crate_prefix))
            .collect::<Vec<_>>();
        quote! {
            #(#fields)*
            Ok(())
        }
    }

    pub fn token_stream_decode<T: ToTokens>(&self, crate_prefix: &T) -> impl ToTokens {
        let fields = self
            .0
            .fields()
            .iter()
            .map(|v| field_info_field_token_stream_decode(v, crate_prefix))
            .collect::<Vec<_>>();
        quote! {
            Ok(Self {
                #(#fields)*
            })
        }
    }

    pub fn full_name(&self) -> String {
        format!("{}", self.0.path())
    }

    pub fn dependencies(&self) -> Vec<MessagePath> {
        self.0.dependencies()
    }

    pub fn get_md5_representation(
        &self,
        hashes: &HashMap<MessagePath, String>,
    ) -> ::std::result::Result<String, ()> {
        self.0.get_md5_representation(hashes).map_err(|_| ())
    }

    pub fn has_header(&self) -> bool {
        self.0.has_header()
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

fn field_info_create_identifier(field_info: &FieldInfo, span: Span) -> Ident {
    if RESERVED_KEYWORDS.contains(field_info.name()) {
        return Ident::new(&format!("{}_", field_info.name()), span);
    }
    Ident::new(field_info.name(), span)
}

fn field_info_field_token_stream<T: ToTokens>(
    field_info: &FieldInfo,
    crate_prefix: &T,
) -> impl ToTokens {
    let datatype = datatype_token_stream(field_info.datatype(), crate_prefix);
    let name = field_info_create_identifier(field_info, Span::call_site());
    match field_info.case() {
        FieldCase::Unit => quote! { pub #name: #datatype, },
        FieldCase::Vector => quote! { pub #name: Vec<#datatype>, },
        FieldCase::Array(l) => quote! { pub #name: [#datatype; #l], },
        FieldCase::Const(_) => quote! {},
    }
}

fn field_info_field_name_eq_and_debug_token_stream(
    field_info: &FieldInfo,
) -> Option<(impl ToTokens, impl ToTokens, impl ToTokens)> {
    let name = field_info_create_identifier(field_info, Span::call_site());
    match field_info.case() {
        FieldCase::Unit | FieldCase::Vector => {
            Some((quote! { #name }, quote! { #name }, quote! { &self.#name }))
        }
        FieldCase::Array(_) => Some((
            quote! { #name },
            quote! { #name[..] },
            quote! { &self.#name.iter().collect::<Vec<_>>() },
        )),
        FieldCase::Const(_) => None,
    }
}

fn field_info_field_default_token_stream<T: ToTokens>(
    field_info: &FieldInfo,
    _crate_prefix: &T,
) -> impl ToTokens {
    let name = field_info_create_identifier(field_info, Span::call_site());
    match field_info.case() {
        FieldCase::Unit | FieldCase::Vector => quote! { #name: Default::default(), },
        FieldCase::Array(l) => {
            let instances = (0..*l).map(|_| quote! {Default::default()});
            quote! { #name: [#(#instances),*], }
        }
        FieldCase::Const(_) => quote! {},
    }
}

fn field_info_field_into_value_token_stream<T: ToTokens>(
    field_info: &FieldInfo,
    _crate_prefix: &T,
) -> impl ToTokens {
    let name = field_info_create_identifier(field_info, Span::call_site());
    let name_str = field_info.name();
    match field_info.case() {
        FieldCase::Unit | FieldCase::Vector | FieldCase::Array(_) => {
            quote! { output.insert(#name_str.into(), src.#name.into()); }
        }
        FieldCase::Const(_) => quote! {},
    }
}

fn field_info_field_from_value_token_stream<T: ToTokens>(
    field_info: &FieldInfo,
    _crate_prefix: &T,
) -> impl ToTokens {
    let name = field_info_create_identifier(field_info, Span::call_site());
    let name_str = field_info.name();
    match field_info.case() {
        FieldCase::Unit | FieldCase::Vector | FieldCase::Array(_) => {
            quote! { #name: src.remove(#name_str).ok_or(())?.try_into()?, }
        }
        FieldCase::Const(_) => quote! {},
    }
}

fn field_info_field_token_stream_encode<T: ToTokens>(
    field_info: &FieldInfo,
    crate_prefix: &T,
) -> impl ToTokens {
    let name = field_info_create_identifier(field_info, Span::call_site());
    match field_info.case() {
        FieldCase::Unit => quote! { self.#name.encode(w.by_ref())?; },
        FieldCase::Vector => match field_info.datatype() {
            DataType::String
            | DataType::Time
            | DataType::Duration
            | DataType::LocalMessage(_)
            | DataType::GlobalMessage(_) => {
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

fn field_info_field_token_stream_decode<T: ToTokens>(
    field_info: &FieldInfo,
    crate_prefix: &T,
) -> impl ToTokens {
    let name = field_info_create_identifier(field_info, Span::call_site());
    match field_info.case() {
        FieldCase::Unit => quote! { #name: #crate_prefix rosmsg::RosMsg::decode(r.by_ref())?, },
        FieldCase::Vector => match field_info.datatype() {
            DataType::String
            | DataType::Time
            | DataType::Duration
            | DataType::LocalMessage(_)
            | DataType::GlobalMessage(_) => {
                quote! { #name: #crate_prefix rosmsg::decode_variable_vec(r.by_ref())?, }
            }
            _ => {
                quote! { #name: #crate_prefix rosmsg::decode_variable_primitive_vec(r.by_ref())?, }
            }
        },
        FieldCase::Array(l) => {
            let lines =
                (0..*l).map(|_| quote! { #crate_prefix rosmsg::RosMsg::decode(r.by_ref())?, });
            quote! { #name: [#(#lines)*], }
        }
        FieldCase::Const(_) => quote! {},
    }
}

fn field_info_const_token_stream<T: ToTokens>(
    field_info: &FieldInfo,
    crate_prefix: &T,
) -> impl ToTokens {
    let value = match field_info.case() {
        FieldCase::Const(ref value) => value,
        _ => return quote! {},
    };
    let name = field_info_create_identifier(field_info, Span::call_site());
    let datatype = datatype_token_stream(field_info.datatype(), crate_prefix);
    let insides = match field_info.datatype() {
        DataType::Bool => {
            let bool_value = if value != "0" {
                quote! { true }
            } else {
                quote! { false }
            };
            quote! { #name: bool = #bool_value }
        }
        DataType::String => quote! { #name: &'static str = #value },
        DataType::Time
        | DataType::Duration
        | DataType::LocalMessage(..)
        | DataType::GlobalMessage(..) => return quote! {},
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

fn datatype_token_stream<T: ToTokens>(data_type: &DataType, crate_prefix: &T) -> impl ToTokens {
    match data_type {
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
        DataType::LocalMessage(ref name) => {
            let name = Ident::new(name, Span::call_site());
            quote! { #name }
        }
        DataType::GlobalMessage(ref message) => {
            let name = Ident::new(message.name(), Span::call_site());
            let pkg = Ident::new(message.package(), Span::call_site());
            quote! { super::#pkg::#name }
        }
    }
}
