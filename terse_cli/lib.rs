extern crate proc_macro;
use proc_macro::TokenStream;

#[proc_macro_attribute]
pub fn command(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let _attr = proc_macro2::TokenStream::from(_attr);
    let item = proc_macro2::TokenStream::from(item);

    terse_cli_lib::command(_attr, item).unwrap().into()
}

#[proc_macro]
pub fn subcommands(item: TokenStream) -> TokenStream {
    terse_cli_lib::subcommands(proc_macro2::TokenStream::from(item))
        .unwrap()
        .into()
}
