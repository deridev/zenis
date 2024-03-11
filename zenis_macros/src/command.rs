use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote};
use syn::{parse2, spanned::Spanned, Block, Error, ItemFn, Lit, Signature};
use zenis_discord::twilight_model::application::command::CommandOptionType;

use crate::{
    argument::Argument,
    common::Result,
    util::{self, capitalize, is_optional, OptionTypeWrapper},
};

pub fn command(attr: TokenStream2, input: TokenStream2) -> Result<TokenStream2> {
    let fun = parse2::<ItemFn>(input)?;

    let ItemFn {
        attrs,
        vis,
        mut sig,
        mut block,
    } = fun;

    if sig.inputs.is_empty() {
        // The function must have at least one argument, which must be an `CommandContext`
        return Err(Error::new(
            sig.inputs.span(),
            "Expected at least CommandContext as a parameter",
        ));
    }

    let name = sig.ident.to_string();

    let command_name = util::get_attribute_argument_literal(&attrs, "name")
        .map(|lit| match lit {
            Lit::Str(str) => str.value(),
            _ => name.clone(),
        })
        .unwrap_or(sig.ident.to_string());

    let description = parse2::<syn::LitStr>(attr)?;

    let args = parse_arguments(&mut sig, &mut block)?;

    let option_tokens = {
        let mut stream = quote!();
        for arg in args.iter() {
            let name = &arg.name.to_string();
            let description = &arg.description;

            let ty: OptionTypeWrapper = arg.ty.clone().into();
            let required = !is_optional(&arg.ty);

            let mut builder_stream = quote!();

            builder_stream.extend(quote!(
                CommandOptionBuilder::new(#name, #description, #ty).set_required(#required)
            ));

            if let Some((min_length, max_length)) = arg.min_max_length {
                builder_stream.extend(quote!(
                    .set_min_max_length(#min_length, #max_length)
                ))
            }

            stream.extend(quote!(.add_option(#builder_stream)))
        }

        stream
    };

    let struct_name = format_ident!("{}Command", capitalize(&name));

    let character_required = util::get_attribute_argument_literal(&attrs, "character_required")
        .map(|lit| match lit {
            Lit::Bool(bool) => bool.value,
            _ => false,
        })
        .unwrap_or_default();

    let city_required = util::get_attribute_argument_literal(&attrs, "city_required")
        .map(|lit| match lit {
            Lit::Bool(bool) => bool.value,
            _ => false,
        })
        .unwrap_or_default();

    // generate the code for the struct and impl
    let expanded = quote! {
        #vis struct #struct_name;
        #[async_trait]
        impl ZenisCommand for #struct_name {
            fn command_config(&self) -> CommandConfig {
                CommandConfig {
                    character_required: #character_required,
                    city_required: #city_required,
                }
            }

            fn build_command(&self, application_id: Id<ApplicationMarker>) -> CommandBuilder {
                CommandBuilder::new(application_id, #command_name, #description)
                    #option_tokens
            }

            async fn run(&self, mut ctx: CommandContext) -> anyhow::Result<()> {
                #block
            }
        }
    };

    Ok(expanded)
}

// Parse arguments and get from ctx the values
pub fn parse_arguments(sig: &mut Signature, block: &mut Block) -> Result<Vec<Argument>> {
    let mut arguments = Vec::new();
    while sig.inputs.len() > 1 {
        arguments.push(Argument::new(sig.inputs.pop().unwrap().into_value())?);
    }

    arguments.reverse();

    let (names, types) = (
        arguments
            .iter()
            .map(|arg| (&arg.name, &arg.ident))
            .collect::<Vec<_>>(),
        arguments.iter().map(|arg| &arg.ty).collect::<Vec<_>>(),
    );

    let original_block = &block;

    let mut extra_argument_block = quote!();
    for ((name, name_identifier), ty) in names.into_iter().zip(types).rev() {
        let is_optional = is_optional(ty);
        let name_string = name.clone().to_string();

        let discord_type: OptionTypeWrapper = ty.clone().into();

        let fn_identifier = match discord_type.0 {
            CommandOptionType::Boolean => format_ident!("get_boolean"),
            CommandOptionType::User => format_ident!("get_user"),
            CommandOptionType::Integer => format_ident!("get_integer"),
            CommandOptionType::Number => format_ident!("get_number"),
            CommandOptionType::String => format_ident!("get_string"),
            _ => unimplemented!(),
        };

        let is_async = discord_type.0 == CommandOptionType::User;

        extra_argument_block.extend(quote! {
            let #name_identifier = ctx.options().#fn_identifier(#name_string)
        });

        if is_async {
            extra_argument_block.extend(quote! { .await? });
        } else {
            extra_argument_block.extend(quote!(?))
        }

        if !is_optional {
            extra_argument_block.extend(quote! { .unwrap() })
        }

        extra_argument_block.extend(quote!(;));
    }

    *block = parse2(quote! {{
        #extra_argument_block
        #original_block
    }})?;

    Ok(arguments)
}
