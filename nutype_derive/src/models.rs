use proc_macro2::{Ident, TokenStream as TokenStream2};
use quote::{quote, ToTokens};

pub use crate::string::models::{StringSanitizer, StringValidator};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypeName(String);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InnerType {
    String,
    Number(NumberType),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NumberType {
    I32,
}

impl ToTokens for InnerType {
    fn to_tokens(&self, token_stream: &mut TokenStream2) {
        match self {
            InnerType::String => {
                quote!(String).to_tokens(token_stream);
            }
            InnerType::Number(number_type) => {
                number_type.to_tokens(token_stream);
            }
        };
    }
}

impl ToTokens for NumberType {
    fn to_tokens(&self, token_stream: &mut TokenStream2) {
        match self {
            Self::I32 => {
                quote!(i32).to_tokens(token_stream);
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypeNameAndInnerType {
    pub type_name: Ident,
    pub inner_type: InnerType,
}

/// Validated model, that represents precisly what needs to be generated.
#[derive(Debug)]
pub enum NewtypeMeta<Sanitizer, Validator> {
    From {
        sanitizers: Vec<Sanitizer>,
    },
    TryFrom {
        sanitizers: Vec<Sanitizer>,
        validators: Vec<Validator>,
    },
}

/// Parsed by not yet validated
#[derive(Debug)]
pub struct RawNewtypeMeta<Sanitizer, Validator> {
    pub sanitizers: Vec<Sanitizer>,
    pub validators: Vec<Validator>,
}
