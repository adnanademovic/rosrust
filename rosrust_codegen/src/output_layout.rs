use crate::msg::Msg;
use proc_macro2::Span;
use quote::ToTokens;
use syn::Ident;

pub struct Layout {
    pub packages: Vec<Package>,
}

impl Layout {
    pub fn token_stream<T: ToTokens>(&self, crate_prefix: &T) -> impl ToTokens {
        let packages = self
            .packages
            .iter()
            .map(|v| v.token_stream(crate_prefix))
            .collect::<Vec<_>>();
        quote! {
            #(#packages)*
        }
    }
}

pub struct Package {
    pub name: String,
    pub messages: Vec<Message>,
    pub services: Vec<Service>,
}

impl Package {
    pub fn token_stream<T: ToTokens>(&self, crate_prefix: &T) -> impl ToTokens {
        let name = Ident::new(&self.name, Span::call_site());
        let messages = self
            .messages
            .iter()
            .map(|v| v.token_stream(crate_prefix))
            .collect::<Vec<_>>();
        let services = self
            .services
            .iter()
            .map(|v| v.token_stream(crate_prefix))
            .collect::<Vec<_>>();
        quote! {
            pub mod #name {
                #(#messages)*
                #(#services)*
            }
        }
    }
}

pub struct Message {
    pub message: Msg,
    pub msg_definition: String,
    pub md5sum: String,
    pub msg_type: String,
}

impl Message {
    pub fn token_stream<T: ToTokens>(&self, crate_prefix: &T) -> impl ToTokens {
        let Message {
            message,
            msg_definition,
            md5sum,
            msg_type,
        } = self;
        let base_message = message.token_stream(crate_prefix);
        let encode_message = message.token_stream_encode(crate_prefix);
        let decode_message = message.token_stream_decode(crate_prefix);
        let name = message.name_ident();
        let header_tokens = message.header_token_stream(crate_prefix);
        quote! {
            #base_message

            impl #crate_prefix Message for #name {
                #[inline]
                fn msg_definition() -> ::std::string::String {
                    #msg_definition.into()
                }

                #[inline]
                fn md5sum() -> ::std::string::String {
                    #md5sum.into()
                }

                #[inline]
                fn msg_type() -> ::std::string::String {
                    #msg_type.into()
                }

                #header_tokens
            }

            impl #crate_prefix rosmsg::RosMsg for #name {
                fn encode<W: ::std::io::Write>(&self, mut w: W) -> ::std::io::Result<()> {
                    #encode_message
                }

                fn decode<R: ::std::io::Read>(mut r: R) -> ::std::io::Result<Self> {
                    #decode_message
                }
            }
        }
    }
}

pub struct Service {
    pub name: String,
    pub md5sum: String,
    pub msg_type: String,
}

impl Service {
    pub fn token_stream<T: ToTokens>(&self, crate_prefix: &T) -> impl ToTokens {
        let Service {
            name,
            md5sum,
            msg_type,
        } = self;
        let name_ident = Ident::new(&name, Span::call_site());
        let req_ident = Ident::new(&format!("{}Req", name), Span::call_site());
        let res_ident = Ident::new(&format!("{}Res", name), Span::call_site());

        quote! {
            #[allow(dead_code,non_camel_case_types,non_snake_case)]
            #[derive(Debug)]
            pub struct #name_ident;

            impl #crate_prefix Message for #name_ident {
                #[inline]
                fn msg_definition() -> ::std::string::String {
                    String::new()
                }

                #[inline]
                fn md5sum() -> ::std::string::String {
                    #md5sum.into()
                }

                #[inline]
                fn msg_type() -> ::std::string::String {
                    #msg_type.into()
                }
            }

            impl #crate_prefix rosmsg::RosMsg for #name_ident {
                fn encode<W: ::std::io::Write>(&self, _w: W) -> ::std::io::Result<()> {
                    Ok(())
                }

                fn decode<R: ::std::io::Read>(_r: R) -> ::std::io::Result<Self> {
                    Ok(Self {})
                }
            }

            impl #crate_prefix ServicePair for #name_ident {
                type Request = #req_ident;
                type Response = #res_ident;
            }
        }
    }
}
