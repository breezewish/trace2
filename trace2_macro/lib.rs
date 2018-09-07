#![recursion_limit = "128"]

extern crate proc_macro;
extern crate proc_macro2;
#[macro_use]
extern crate syn;
#[macro_use]
extern crate quote;

mod codegen;
mod config;
mod folder;

#[proc_macro_attribute]
pub fn trace2(
    args: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    folder::Folder::fold(args.into(), input.into()).into()
}
