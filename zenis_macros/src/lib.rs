mod argument;
mod attr;
mod command;
mod common;
mod util;

use common::Result;
use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;

fn extract(res: Result<TokenStream2>) -> TokenStream {
    match res {
        Ok(s) => s,
        Err(why) => why.to_compile_error(),
    }
    .into()
}

#[proc_macro_attribute]
pub fn command(attrs: TokenStream, input: TokenStream) -> TokenStream {
    extract(command::command(attrs.into(), input.into()))
}

use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput};

#[proc_macro_derive(List)]
pub fn list_derive(input: TokenStream) -> TokenStream {
    // Parse the input tokens into a syntax tree
    let input = parse_macro_input!(input as DeriveInput);

    // Check that the input is an enum
    let data = match &input.data {
        Data::Enum(data) => data.clone(),
        _ => panic!("List can only be derived for enums"),
    };

    // Get the name of the enum
    let name = input.ident;

    // Build a list of enum variants
    let variants = data.variants.iter().map(|v| &v.ident);
    let num_variants = variants.len();
    let list = quote!([#(Self::#variants),*]);

    // Generate the output code
    let output = quote! {
        impl #name {
            pub const LIST: [Self; #num_variants] = #list;
        }
    };

    // Return the output as a token stream
    output.into()
}
