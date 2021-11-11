#[macro_use]
mod utils;
mod attributes;
mod constants;
mod structures;

extern crate proc_macro;
use proc_macro::TokenStream;

use attributes::*;
use constants::*;
use structures::*;
use utils::*;

use quote::quote;
use syn::spanned::Spanned;
use syn::{parse_macro_input, parse_quote, Error, Ident, Lit};

#[proc_macro_attribute]
pub fn slash_command(attr: TokenStream, input: TokenStream) -> TokenStream {
    let mut fun = parse_macro_input!(input as SlashCommandFun);

    let _name = if !attr.is_empty() {
        parse_macro_input!(attr as Lit).to_str()
    } else {
        fun.name.to_string().trim_start_matches("r#").into()
    };

    let mut options = Options::new();

    for attr in &fun.attributes {
        let span = attr.span();
        let values = propagate_err!(parse_values(attr));

        let name = values.name.to_string();

        match name.as_str() {
            "description" => {
                let line: String = propagate_err!(attributes::parse(values));
                utils::append_line(&mut options.description, line);
            }
            "guild" => {
                let guild_id: u64 = propagate_err!(attributes::parse(values));
                options.guild_ids.push(guild_id);
            }
            "default_permission" => {
                let default_permission: bool = propagate_err!(attributes::parse(values));
                options.default_permission = default_permission;
            }
            "permission" => {
                let permission_name: Ident = propagate_err!(attributes::parse(values));
                options.permissions.push(permission_name);
            }
            "option" => {
                let option_name: Ident = propagate_err!(attributes::parse(values));
                options.options.push(option_name);
            }
            _ => {
                return Error::new(span, format_args!("invalid attribute: {:?}", attr))
                    .to_compile_error()
                    .into();
            }
        }
    }

    let Options {
        description,
        guild_ids,
        default_permission,
        permissions,
        options,
    } = options;

    propagate_err!(create_declaration_validations(&mut fun));

    let res = parse_quote!(serenity::Result<()>);
    create_return_type_validation(&mut fun, res);

    let visibility = fun.visibility;
    let name = fun.name.clone();
    // let options = name.with_suffix(COMMAND_OPTIONS);
    let body = fun.body;
    let ret = fun.ret;

    let n = name.with_suffix(COMMAND);

    let cooked = fun.cooked.clone();

    let slash_command_path = quote!(serenity_slash_command_framework::SlashCommand);

    let async_trait_path = quote!(serenity::async_trait);
    let create_application_command_path = quote!(serenity::builder::CreateApplicationCommand);
    let create_application_command_permissions_data_path =
        quote!(serenity::builder::CreateApplicationCommandPermissionsData);

    let guild_ids = if guild_ids.is_empty() {
        quote! { None }
    } else {
        quote! { Some(&[#(#guild_ids),*]) }
    };

    let mut create_function = quote! {
        c.default_permission(#default_permission)
    };

    if let Some(desc) = description.0 {
        create_function = quote! {
            #create_function
                .description(#desc)
        };
    }

    let create_function = quote! {
        fn create(c: &mut #create_application_command_path) -> &mut #create_application_command_path {
            #create_function
                #(.create_option(|o| #options::apply(o)))*
        }
    };

    let mut permission_function = quote! {
        #(#permissions::apply(p);)*
    };

    if !permission_function.is_empty() {
        permission_function = quote! {
            fn permissions(p: &mut #create_application_command_permissions_data_path) -> &mut #create_application_command_permissions_data_path {
                #permission_function
                p
            }
        };
    }

    populate_fut_lifetimes_on_refs(&mut fun.args);
    let args = fun.args;

    (quote! {
        #(#cooked)*
        #[allow(missing_docs)]
        #visibility struct #n;

        #[#async_trait_path]
        #[allow(missing_docs)]
        impl #slash_command_path for #n {
            const NAME: &'static str = stringify!(#name);
            const GUILDS: Option<&'static [u64]> = #guild_ids;

            #create_function

            #permission_function

            #(#cooked)*
            async fn callback(#(#args),*) -> #ret {
                #(#body)*
            }
        }
    })
    .into()
}
