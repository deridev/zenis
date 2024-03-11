use proc_macro2::TokenStream;
use quote::ToTokens;
use syn::{spanned::Spanned, Attribute, Error, Ident, Lit, Meta, NestedMeta, Path};

use crate::common::Result;

/// Values an [attr](self::Attr) can have
#[derive(Debug, Clone)]
pub enum Value {
    /// An identifier
    Ident(Ident),
    /// A literal value
    Lit(Lit),
}

impl ToTokens for Value {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            Value::Ident(ident) => ident.to_tokens(tokens),
            Value::Lit(lit) => lit.to_tokens(tokens),
        }
    }
}

/// A simplified version of an attribute
#[derive(Debug, Clone)]
pub struct Attr {
    /// The path of this attribute
    ///
    /// e.g.: In `#[name = "some"]`, the part of `name` is the path of the attribute
    pub path: Path,
    /// The data type the attribute can have
    pub values: Vec<Value>,
}

impl Attr {
    /// Creates a new [attr](self::Attr)
    pub fn new(path: Path, values: Vec<Value>) -> Self {
        Self { path, values }
    }

    pub fn parse_one(attribute: &Attribute) -> Result<Self> {
        let meta = attribute.parse_meta()?;

        match meta {
            Meta::Path(p) => Ok(Attr::new(p, Vec::new())),
            Meta::List(l) => {
                let path = l.path;
                let values = l
                    .nested
                    .into_iter()
                    .map(|m| match m {
                        NestedMeta::Lit(lit) => Ok(Value::Lit(lit)),
                        NestedMeta::Meta(m) => match m {
                            Meta::Path(p) => Ok(Value::Ident(p.get_ident().unwrap().clone())),
                            _ => Err(Error::new(
                                m.span(),
                                "Nested lists or name values are not supported",
                            )),
                        },
                    })
                    .collect::<Result<Vec<_>>>()?;

                Ok(Attr::new(path, values))
            }
            Meta::NameValue(nv) => Ok(Attr::new(nv.path, vec![Value::Lit(nv.lit)])),
        }
    }

    /// Executes the given function into the [attr](self::Attr)
    pub fn parse_value<T>(&self, f: impl FnOnce(&Value) -> Result<T>) -> Result<T> {
        if self.values.is_empty() {
            return Err(Error::new(self.span(), "Attribute input must not be empty"));
        }

        if self.values.len() > 1 {
            return Err(Error::new(
                self.span(),
                "Attribute input must not exceed more than one argument",
            ));
        }

        f(&self.values[0])
    }

    pub fn parse_two_u16(&self) -> Result<(u16, u16)> {
        if self.values.is_empty() {
            return Err(Error::new(self.span(), "Attribute input must not be empty"));
        }

        if self.values.len() != 2 {
            return Err(Error::new(
                self.span(),
                "Attribute input must be 2 arguments.",
            ));
        }

        let values = [self.values[0].clone(), self.values[1].clone()].map(|value| match value {
            Value::Lit(Lit::Int(lit)) => match lit.base10_parse::<u16>() {
                Ok(value) => Ok(value),
                Err(e) => Err(Error::new(lit.span(), e.to_string())),
            },
            _ => Err(Error::new(
                self.values[0].span(),
                "Attribute input must be a u16 literal",
            )),
        });

        Ok((values[0].clone()?, values[1].clone()?))
    }

    /// Parses the first literal into a string, returning an error if this attribute does not have any
    /// of them or has identifiers instead of literals
    pub fn parse_string(&self) -> Result<String> {
        self.parse_value(|value| {
            Ok(match value {
                Value::Lit(Lit::Str(s)) => s.value(),
                _ => return Err(Error::new(value.span(), "Argument must be a string")),
            })
        })
    }
}

impl ToTokens for Attr {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Attr { path, values } = self;

        tokens.extend(if values.is_empty() {
            quote::quote!(#[#path])
        } else {
            quote::quote!(#[#path(#(#values)*,)])
        });
    }
}

pub fn parse_attribute(attribute: &Attribute) -> Result<Attr> {
    Attr::parse_one(attribute)
}
