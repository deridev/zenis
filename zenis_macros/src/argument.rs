use syn::{spanned::Spanned, Attribute, Error, FnArg, Type};

use crate::{
    attr::{self, Attr},
    common::Result,
    util,
};

#[derive(Debug)]
pub struct Argument {
    pub ident: syn::Ident,
    pub name: String,
    pub ty: Type,
    pub description: String,
    pub min_max_length: Option<(u16, u16)>,
}

impl Argument {
    pub fn new(arg: FnArg) -> Result<Self> {
        let pat = util::get_pat(&arg)?;
        let ident = util::get_ident(&pat.pat)?;
        let ty = *pat.ty.clone();

        let rename = pat
            .attrs
            .iter()
            .map(extract_attribute_renamed)
            .collect::<Result<Vec<_>>>()
            .into_iter()
            .flatten()
            .flatten()
            .collect::<Vec<_>>();

        let name = rename.first().cloned().unwrap_or(ident.to_string());

        let mut descriptions = pat
            .attrs
            .iter()
            .map(extract_attribute_description)
            .collect::<Result<Vec<_>>>()
            .into_iter()
            .flatten()
            .flatten()
            .collect::<Vec<_>>();

        let min_max_length = pat
            .attrs
            .iter()
            .map(extract_attribute_min_max_length)
            .collect::<Vec<_>>()
            .into_iter()
            .flatten()
            .flatten()
            .collect::<Vec<_>>();

        if descriptions.len() > 1 {
            // We only want a single description attribute
            return Err(Error::new(
                arg.span(),
                "Only allowed a single description attribute",
            ));
        } else if descriptions.is_empty() {
            // Description attribute is required
            return Err(Error::new(arg.span(), "Description attribute is required"));
        }

        Ok(Self {
            ident,
            name,
            description: descriptions.remove(0),
            ty,
            min_max_length: min_max_length.first().cloned(),
        })
    }
}

/// Executes the given closure into an [attr](crate::attr::Attr)
fn exec<F, R>(attr: &Attribute, fun: F) -> Result<R>
where
    F: FnOnce(Attr) -> Result<R>,
{
    fun(attr::parse_attribute(attr)?)
}

fn extract_attribute_renamed(attr: &Attribute) -> Result<Option<String>> {
    exec(attr, |parsed| {
        if parsed.path.is_ident("rename") {
            Ok(Some(parsed.parse_string()?))
        } else {
            Ok(None)
        }
    })
}

fn extract_attribute_description(attr: &Attribute) -> Result<Option<String>> {
    exec(attr, |parsed| {
        if parsed.path.is_ident("description") {
            Ok(Some(parsed.parse_string()?))
        } else {
            Ok(None)
        }
    })
}

fn extract_attribute_min_max_length(attr: &Attribute) -> Result<Option<(u16, u16)>> {
    exec(attr, |parsed| {
        if parsed.path.is_ident("min_max_length") {
            Ok(Some(parsed.parse_two_u16()?))
        } else {
            Ok(None)
        }
    })
}
