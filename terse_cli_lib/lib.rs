#![feature(let_chains)]

use convert_case::{Case, Casing};
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use some_error::some_error;
use syn::parse::{Parse, ParseStream};
use syn::{bracketed, parse2, Attribute, Ident, ItemFn, Token};

/// don't use this directly, use apps/macros instead
#[some_error]
pub fn command(
    _attr: TokenStream,
    item: TokenStream,
) -> Result<TokenStream, InvalidCliArgumentError + InvalidSubcommandFunctionError> {
    let input = parse2::<ItemFn>(item).map_err(|err| InvalidSubcommandFunctionError {
        message: "Failed to parse function",
        err,
    })?;
    let fn_name = &input.sig.ident;
    let args = &input.sig.inputs;
    let (fields, arg_names): (Vec<_>, Vec<_>) = args
        .iter()
        .map(|arg| match arg {
            syn::FnArg::Typed(syn::PatType { pat, ty, .. }) => {
                Ok((quote! { #[arg(long)] #pat: #ty }, quote! { #pat }))
            }
            _ => Err(InvalidCliArgumentError(format!(
                "Invalid cli argument: {}",
                arg.to_token_stream()
            ))),
        })
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .unzip();

    let expanded = quote! {
        mod #fn_name {
            use super::#fn_name;
            use clap::{command, Parser};

            #[derive(Parser)]
            #[command(version, about, long_about = None)]
            pub struct Args {
                #(#fields),*
            }
            pub fn run(Args { #(#arg_names),* }: Args) {
                let result = #fn_name(#(#arg_names),*);
                println!("{}", result);
            }
        }
        #input
    };
    Ok(expanded)
}

/// don't use this directly, use apps/macros instead
#[some_error]
pub fn subcommands(
    item: TokenStream,
) -> Result<TokenStream, InvalidIdentifierError + InvalidIdentifierListError> {
    let MergeSubcommandsInput {
        cli_ident,
        subcommands,
    } = parse2::<MergeSubcommandsInput>(item).map_err(|err| InvalidIdentifierListError {
        message: "subcommands only accepts lists of identifiers",
        err,
    })?;
    let idents: Vec<_> = subcommands
        .into_iter()
        .map(|subcommand| subcommand.ident)
        .collect();

    let match_arms = idents.iter().map(|ident| {
        let cmd_name = get_command_name(ident);
        quote! {
            Subcommands::#cmd_name(args) => #ident::run(args)
        }
    });
    let command_enum_fields = idents.iter().map(|ident| {
        let cmd_name = get_command_name(ident);
        quote! { #cmd_name(#ident::Args) }
    });
    let idents_tokens = idents.iter().map(|ident| ident.to_token_stream());

    let expanded = quote! {
        mod #cli_ident {
            use super::{#(#idents_tokens),*};
            use clap::{command, Parser, Subcommand};

            #[derive(Subcommand)]
            pub enum Subcommands {
                #(#command_enum_fields),*
            }
            #[derive(Parser)]
            #[command(version, about, long_about = None)]
            pub struct Args {
                #[command(subcommand)]
                command: Subcommands,
            }
            pub fn run(Args { command }: Args) {
                match command {
                    #(#match_arms),*
                };
            }
        }
    };

    Ok(expanded)
}

#[allow(dead_code)]
struct Subcommand {
    attrs: Vec<Attribute>,
    ident: Ident,
}

struct MergeSubcommandsInput {
    cli_ident: Ident,
    subcommands: Vec<Subcommand>,
}

impl Parse for Subcommand {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let attrs = input.call(Attribute::parse_outer)?;
        let ident: Ident = input.parse()?;
        Ok(Subcommand { attrs, ident })
    }
}

impl Parse for MergeSubcommandsInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let cli_ident: Ident = input.parse()?;
        input.parse::<Token![,]>()?;
        let content;
        bracketed!(content in input);
        let subcommands: syn::punctuated::Punctuated<Subcommand, Token![,]> =
            content.parse_terminated(Subcommand::parse, Token![,])?;
        Ok(MergeSubcommandsInput {
            cli_ident,
            subcommands: subcommands.into_iter().collect(),
        })
    }
}

fn get_command_name(func_name: &Ident) -> Ident {
    Ident::new(
        &func_name.to_string().to_case(Case::UpperCamel),
        func_name.span(),
    )
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub struct InvalidIdentifierError(&'static str);

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct InvalidIdentifierListError {
    message: &'static str,
    err: syn::Error,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct InvalidSubcommandFunctionError {
    message: &'static str,
    err: syn::Error,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct InvalidCliArgumentError(String);
