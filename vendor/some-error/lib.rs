//! A library for allowing Anonymous Sum Types in the error of the returned result.
//! The function this is attached to must return a type in the form of `Result<T, E1 + E2 + ...>`.
//!
//! ## Example:
//!
//! ```rust
//! use std::io;
//! use some_error::*;
//!
//! #[derive(Debug, Clone, Copy)]
//! struct NotZeroError(u32);
//!
//! #[some_error]
//! fn my_func() -> Result<(), io::Error + NotZeroError>{
//!     let x = 3;
//!     if x != 0 {
//!         Err(NotZeroError(x))?;
//!     }
//!
//!     Ok(())
//! }
//!
//! fn main() {
//!     match my_func() {
//!         Ok(_) => {
//!             println!("Worked ok!");
//!         }
//!         Err(my_func::NotZeroError(NotZeroError(x))) => {
//!             println!("{} is not zero!!", x);
//!         }
//!         Err(my_func::io::Error(io_err)) => {
//!             println!("io error: {:?}", io_err);
//!         }
//!     }
//! }
//! ```
use proc_macro::TokenStream;
use quote::quote;

#[derive(Copy, Clone)]
enum ResultPos {
    Ok = 0,
    Err = 1,
}

fn get_result_type(ret: &mut syn::ReturnType, pos: ResultPos) -> &mut syn::Type {
    match ret {
        syn::ReturnType::Type(_, ty) => match ty.as_mut() {
            syn::Type::Path(syn::TypePath { path, .. }) => {
                let end = path.segments.iter_mut().last().unwrap();

                if end.ident == "Result" {
                    match &mut end.arguments {
                        syn::PathArguments::AngleBracketed(args) => {
                            if args.args.len() != 2 {
                                panic!("Return type must be `Result<T, E>`")
                            }

                            let err = args.args.iter_mut().nth(pos as usize).unwrap();

                            match err {
                                syn::GenericArgument::Type(ref mut err) => err,
                                _ => panic!("Return type must be `Result<T, E>`"),
                            }
                        }
                        _ => panic!("Return type must be `Result<T, E>`"),
                    }
                } else {
                    panic!("Return type must be `Result<T, E>`")
                }
            }
            _ => panic!("Return type must be `Result<T, E>`"),
        },
        syn::ReturnType::Default => panic!("Return type must be `Result<T, E>`"),
    }
}

fn get_ok_type(ret: &mut syn::ReturnType) -> syn::Type {
    get_result_type(ret, ResultPos::Ok).clone()
}

fn get_err_type(ret: &mut syn::ReturnType) -> &mut syn::Type {
    get_result_type(ret, ResultPos::Err)
}

fn with_first_letter_uppercase(ident: String) -> String {
    ident
        .chars()
        .enumerate()
        .map(|(i, first_char)| if i == 0 { first_char.to_ascii_uppercase() } else { first_char })
        .collect()
}

fn path_to_normalized_path(path: &syn::Path) -> syn::Ident {
    let normal_path: String = path
        .segments
        .iter()
        .map(|seg| seg.ident.to_string())
        .map(with_first_letter_uppercase)
        .collect();

    syn::Ident::new(&normal_path, proc_macro2::Span::call_site())
}

fn generate_error_modules(variants: &[syn::Ident], types: &[&syn::Path]) -> impl quote::ToTokens {
    let modules = types.iter().zip(variants.iter()).map(|(path, variant)| {
        let segs: Vec<&_> = path.segments.iter().collect();
        let (last, segs) = segs.split_last().unwrap();

        let supers = std::iter::repeat_with(|| quote!(super)).take(segs.len());
        let supers = quote!(
            #(#supers::)*
        );

        let mut modules = quote!(
            pub use #supers Error::#variant as #last;
        );

        for seg in segs.iter().rev() {
            modules = quote!(
                pub mod #seg {
                    #modules
                }
            );
        }

        modules
    });

    quote!(
        #(
            #modules
        )*
    )
}

fn generate_into_impls(variants: &[syn::Ident], types: &[&syn::Path]) -> impl quote::ToTokens {
    let modules = types.iter().zip(variants.iter()).map(|(path, variant)| {
        quote!(
            impl ::core::convert::From<super::#path> for Error {
                fn from(val: super::#path) -> Error {
                    Error::#variant(val)
                }
            }
        )
    });

    quote!(
        #(
            #modules
        )*
    )
}

fn generate_error_enum(err: &mut syn::Type) -> impl quote::ToTokens {
    let is_lifetime_bound = |x: &syn::TypeParamBound| {
        if let syn::TypeParamBound::Lifetime(_) = x {
            true
        } else {
            false
        }
    };
    let (variant_names, types): (Vec<_>, Vec<_>) = match err {
        syn::Type::TraitObject(trait_obj) => {
            if trait_obj.bounds.iter().any(is_lifetime_bound) {
                panic!("Lifetime bounds are not allowed in anonymous sum type")
            } else {
                trait_obj
                    .bounds
                    .iter()
                    .filter_map(|x| match x {
                        syn::TypeParamBound::Trait(syn::TraitBound { path, .. }) => Some(path),
                        _ => None,
                    })
                    .map(|path| (path_to_normalized_path(path), path))
                    .unzip()
            }
        }
        _ => panic!("Return type must be in form of `Result<T, E1 + E2 + ...>`"),
    };

    let error_modules = generate_error_modules(&variant_names, &types);
    let into_impls = generate_into_impls(&variant_names, &types);

    quote!(
        #[derive(Debug, Clone)]
        pub enum Error {
            #(
                #variant_names(super::#types)
            ),*
        }

        #error_modules

        #into_impls
    )
}

/// A Macro for allowing Anonymous Sum Types in the error of the returned result.
/// The function this is attached to must return a type in the form of `Result<T, E1 + E2 + ...>`.
///
/// ## Example:
///
/// ```rust
/// use std::io;
/// use some_error::*;
///
/// #[derive(Debug, Clone, Copy)]
/// struct NotZeroError(u32);
///
/// #[some_error]
/// fn my_func() -> Result<(), io::Error + NotZeroError>{
///     let x = 3;
///     if x != 0 {
///         Err(NotZeroError(x))?;
///     }
///
///     Ok(())
/// }
///
/// fn main() {
///     match my_func() {
///         Ok(_) => {
///             println!("Worked ok!");
///         }
///         Err(my_func::NotZeroError(NotZeroError(x))) => {
///             println!("{} is not zero!!", x);
///         }
///         Err(my_func::io::Error(io_err)) => {
///             println!("io error: {:?}", io_err);
///         }
///     }
/// }
/// ```
///
/// ## More Info
///
/// * The type of the anonymous sum type can be referenced via `function_name::Error`.
/// * All variants of the anonymous sum type can be accessed via `function_name::<path>`
///     * For example if you use the return type `Result<(), fmt::Error + i32>` the variants will
///     be named `function_name::fmt::Error` and `function_name::i32` and can be used as patterns
///     to match against (see the above example)
#[proc_macro_attribute]
pub fn some_error(_: TokenStream, contents: TokenStream) -> TokenStream {
    let mut function = syn::parse_macro_input!(contents as syn::ItemFn);

    let vis = function.vis.clone();
    let ident = function.sig.ident.clone();
    let ok_type = get_ok_type(&mut function.sig.output);
    let err_type = get_err_type(&mut function.sig.output);

    let error_enum = generate_error_enum(err_type);

    function.sig.output = syn::parse_quote!(
        -> Result<#ok_type, #ident::Error>
    );

    quote!(
        #vis mod #ident {
            #error_enum
        }

        #function
    )
    .into()
}
