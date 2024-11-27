#![feature(let_chains)]

use convert_case::{Case, Casing};
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use some_error_fork::some_error;
use syn::parse::{Parse, ParseStream};
use syn::{bracketed, parse2, Attribute, Ident, ItemFn, Token};

fn extract_docs(a: &[Attribute]) -> impl Iterator<Item = &Attribute> {
    a.iter().filter(|a| a.path().is_ident("doc"))
}

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

    let docs = extract_docs(&input.attrs);

    let expanded = quote! {
        mod #fn_name {
            use super::#fn_name;
            use clap::{command, Parser};

            #[derive(Parser)]
            #[command(version, about, long_about = None)]
            #(#docs)*
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
        sub_doc,
        cli_ident,
        subcommands,
    } = parse2::<MergeSubcommandsInput>(item).map_err(|err| InvalidIdentifierListError {
        message: "subcommands only accepts lists of identifiers",
        err,
    })?;

    let match_arms = subcommands.iter().map(|sc| {
        let ident = &sc.ident;
        let cmd_name = get_command_name(ident);
        quote! {
            Subcommands::#cmd_name(args) => #ident::run(args)
        }
    });
    let command_enum_fields = subcommands.iter().map(|sc| {
        let docs = extract_docs(&sc.attrs);
        let ident = &sc.ident;
        let cmd_name = get_command_name(ident);
        quote! { 
            #(#docs)*
            #cmd_name(#ident::Args)
        }
    });
    let idents_tokens = subcommands.iter().map(|sc| sc.ident.to_token_stream());
    let sub_doc_mod = sub_doc.clone();

    let expanded = quote! {
        #(#sub_doc_mod)*
        mod #cli_ident {
            use super::{#(#idents_tokens),*};
            use clap::{command, Parser, Subcommand};

            #[derive(Subcommand)]
            #(#sub_doc)*
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
    sub_doc: Vec<Attribute>,
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
        // parse any doc comments attached to the cli_ident
        let sub_doc = Attribute::parse_outer(input)
            .unwrap_or_default()
            .into_iter()
            .filter(|attr| attr.path().is_ident("doc"))
            .collect::<Vec<_>>();

        let cli_ident: Ident = input.parse()?;
        input.parse::<Token![,]>()?;
        let content;
        bracketed!(content in input);
        let subcommands: syn::punctuated::Punctuated<Subcommand, Token![,]> =
            content.parse_terminated(Subcommand::parse, Token![,])?;
        Ok(MergeSubcommandsInput {
            sub_doc,
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
