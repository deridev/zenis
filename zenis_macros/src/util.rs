use proc_macro2::TokenStream;
use quote::{format_ident, quote, ToTokens};
use syn::{spanned::Spanned, Attribute, Error, FnArg, Ident, Pat, PatType, Path, Type};
use zenis_discord::twilight_model::application::command::CommandOptionType;

use crate::{
    attr::{Attr, Value},
    common::Result,
};

pub fn capitalize(string: &str) -> String {
    let string = string.to_string();
    let mut chars = string.chars();
    let first = chars.next();

    let collected = chars.collect::<String>();

    format!(
        "{}{collected}",
        first.expect("Failed at `capitalize`").to_ascii_uppercase()
    )
}

pub fn get_attribute_argument_literal(args: &[Attribute], name: &str) -> Option<syn::Lit> {
    let attrs = args
        .iter()
        .map(|a| Attr::parse_one(a).unwrap())
        .collect::<Vec<_>>();

    for attribute in attrs {
        if attribute.path.segments.last().unwrap().ident == name {
            let value = attribute.values.first().unwrap();
            if let Value::Lit(lit) = value {
                return Some(lit.clone());
            }
        }
    }

    None
}

/// Gets the path of the given type
pub fn get_path(t: &Type, allow_references: bool) -> Result<&Path> {
    match t {
        // If the type is actually a path, just return it
        Type::Path(p) => Ok(&p.path),
        // If the type is a reference, call this function recursively until we get the path
        Type::Reference(r) => {
            if allow_references {
                get_path(&r.elem, allow_references)
            } else {
                Err(Error::new(r.span(), "Reference not allowed"))
            }
        }
        _ => Err(Error::new(
            t.span(),
            "parameter must be a path to a context type",
        )),
    }
}

pub fn get_pat(arg: &FnArg) -> Result<&PatType> {
    match arg {
        FnArg::Typed(t) => Ok(t),
        _ => Err(Error::new(
            arg.span(),
            "`self` parameter is not allowed here",
        )),
    }
}

pub fn is_optional(ty: &Type) -> bool {
    match &ty {
        Type::Path(path) => path
            .path
            .segments
            .first()
            .map_or(false, |segment| segment.ident == "Option"),
        _ => false,
    }
}

/// Gets the identifier of the given pattern
pub fn get_ident(p: &Pat) -> Result<Ident> {
    match p {
        Pat::Ident(pi) => Ok(pi.ident.clone()),
        _ => Err(Error::new(p.span(), "parameter must have an identifier")),
    }
}

pub struct OptionTypeWrapper(pub CommandOptionType);

impl From<Type> for OptionTypeWrapper {
    fn from(value: Type) -> Self {
        let value = get_inner_type(&value);
        let path = get_path(value, false).unwrap();
        let segment = path.segments.last().unwrap();

        match segment.ident.to_string().as_str() {
            "String" => Self(CommandOptionType::String),
            "i64" => Self(CommandOptionType::Integer),
            "f64" => Self(CommandOptionType::Number),
            "bool" => Self(CommandOptionType::Boolean),
            "User" => Self(CommandOptionType::User),
            _ => panic!("Unexpected type: {}", segment.ident),
        }
    }
}

impl ToTokens for OptionTypeWrapper {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let ty = match self.0 {
            CommandOptionType::String => format_ident!("String"),
            CommandOptionType::Integer => format_ident!("Integer"),
            CommandOptionType::Number => format_ident!("Number"),
            CommandOptionType::Boolean => format_ident!("Boolean"),
            CommandOptionType::User => format_ident!("User"),
            _ => unimplemented!(),
        };

        tokens.extend(quote!(CommandOptionType::#ty));
    }
}

fn get_inner_type(ty: &Type) -> &Type {
    if let Type::Path(path) = ty {
        if let Some(first) = path.path.segments.first() {
            if first.ident == "Option" {
                if let syn::PathArguments::AngleBracketed(args) = &first.arguments {
                    if let Some(syn::GenericArgument::Type(inner_ty)) = args.args.first() {
                        return inner_ty;
                    } else {
                        panic!("Unexpected argument: {:?}", first.arguments);
                    }
                } else {
                    panic!("Unexpected arguments: {:?}", first.arguments);
                }
            }
        }
    }

    ty
}
