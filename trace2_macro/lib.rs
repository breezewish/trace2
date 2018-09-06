#![recursion_limit="128"]

extern crate proc_macro;
extern crate proc_macro2;
extern crate syn;
#[macro_use]
extern crate quote;

use proc_macro::TokenStream;

use syn::fold::Fold;
use quote::ToTokens;

#[proc_macro_attribute]
pub fn trace2(_args: TokenStream, input: TokenStream) -> TokenStream {
    let folded = fold(input).into();
    println!("{}", folded);
    folded
}

fn fold(input: TokenStream) -> proc_macro2::TokenStream {
    let mut folder = Folder;
    // Try to parse as mod {}
    let body: Result<syn::ItemMod, _> = syn::parse(input.clone());
    if let Ok(body) = body {
        return folder.fold_item_mod(body).into_token_stream()
    }

    // Try to parse as fn()
    let body: Result<syn::ItemFn, _> = syn::parse(input.clone());
    if let Ok(body) = body {
        return folder.fold_item_fn(body).into_token_stream()
    }

    // Try to parse as impl {}
    let body: Result<syn::ItemImpl, _> = syn::parse(input.clone());
    if let Ok(body) = body {
        return folder.fold_item_impl(body).into_token_stream()
    }

    panic!("Invalid attribute position, only supports function, impl and mod.");
}

fn extract_printable_args<'a>(pat: &'a syn::Pat, extract_target: &mut Vec<&'a syn::Ident>) {
    match pat {
        syn::Pat::Wild(_) |
        syn::Pat::Path(_) |
        syn::Pat::Box(_) |
        syn::Pat::Ref(_) |
        syn::Pat::Lit(_) |
        syn::Pat::Range(_) |
        syn::Pat::Verbatim(_) |
        syn::Pat::Macro(_) => {
            panic!("Unexpected argument pattern")
        },
        syn::Pat::Ident(ref ident) => {
            if ident.ident.to_string() != "self" {
                extract_target.push(&ident.ident);
            }
        }
        syn::Pat::TupleStruct(ref tuple_struct) => {
            for pat in tuple_struct.pat.front.iter() {
                extract_printable_args(pat, extract_target);
            }
            for pat in tuple_struct.pat.back.iter() {
                extract_printable_args(pat, extract_target);
            }
        }
        syn::Pat::Tuple(ref tuple) => {
            for pat in tuple.front.iter() {
                extract_printable_args(pat, extract_target);
            }
            for pat in tuple.back.iter() {
                extract_printable_args(pat, extract_target);
            }
        }
        syn::Pat::Struct(ref structure) => {
            for pat in structure.fields.iter() {
                extract_printable_args(&*pat.pat, extract_target);
            }
        }
        syn::Pat::Slice(ref slice) => {
            for pat in slice.front.iter() {
                extract_printable_args(pat, extract_target);
            }
            if let Some(ref pat) = slice.middle {
                extract_printable_args(&*pat, extract_target);
            }
            for pat in slice.back.iter() {
                extract_printable_args(pat, extract_target);
            }
        }
    }
}

fn build_begin_trace_statement(fn_decl: &syn::FnDecl, fn_ident: &syn::Ident) -> proc_macro2::TokenStream {
    // build statements like:
    // ```
    // trace!("{} {}::foo(arg1: {:?}, arg2: {:?})", ">".repeat(..), module_path!(), arg1, arg2);
    // ```

    let mut args = vec![];
    for fn_arg in fn_decl.inputs.iter() {
        match fn_arg {
            syn::FnArg::SelfRef(_) | syn::FnArg::SelfValue(_) => {
                // ignore self arg
            },
            syn::FnArg::Captured(ref arg) => {
                extract_printable_args(&arg.pat, &mut args);
            },
            syn::FnArg::Inferred(ref arg_pat) => {
                extract_printable_args(arg_pat, &mut args);
            },
            syn::FnArg::Ignored(_) => {
                // ignore ignored arg
            },
        }
    }

    let format_args = args
        .iter()
        .map(|arg_ident| format!("{}: {{:?}}", arg_ident))
        .collect::<Vec<_>>()
        .join(", ");
    let format = format!("{{}} {{}}::{}({})", fn_ident, format_args);

    quote! {
        trace!(#format, ">".repeat(__level * 4), module_path!(), #(#args),*)
    }
}

fn build_end_trace_statement(fn_ident: &syn::Ident) -> proc_macro2::TokenStream {
    // build statements like:
    // ```
    // trace!("{} {}::foo = {:?}", "<".repeat(..), module_path!(), __ret);
    // ```

    let format = format!("{{}} {{}}::{} = {{:?}}", fn_ident);

    quote! {
        trace!(#format, "<".repeat(__level * 4), module_path!(), __ret)
    }
}

fn build_block(decl: &syn::FnDecl, ident: &syn::Ident, block: &syn::Block) -> proc_macro2::TokenStream {
    // We receive:
    // ```
    // (pub) fn foo<T>(arg1: T, arg2: foo) -> bool where T: bar {
    //     ...
    // }
    // ```
    //
    // Transform it into:
    // ```
    // (pub) fn foo<T>(&self, arg1: T, arg2: foo) -> bool where T: bar {
    //     trace!("{} foo(arg1: {:?}, arg2: {:?})", ">".repeat(..), arg1, arg2);
    //     let mut __inner = move || {
    //         ...
    //     };
    //     let __ret = __inner();
    //     trace!("{} foo = {:?}", ">".repeat(..), __ret);
    //     __ret
    // }
    // ```

    let begin_trace = build_begin_trace_statement(decl, ident);
    let end_trace = build_end_trace_statement(ident);
    quote! {
        {
            use trace2;
            trace2::FUNC_CALL_LEVEL.with(|level| {
                let mut __level = level.get();
                __level = __level.saturating_add(1);
                level.set(__level);
                #begin_trace;
            });
            let mut __inner = move || #block;
            let __ret = __inner();
            trace2::FUNC_CALL_LEVEL.with(|level| {
                let mut __level = level.get();
                #end_trace;
                __level = __level.saturating_sub(1);
                level.set(__level);
            });
            __ret
        }
    }
}

struct Folder;

impl Fold for Folder {
    fn fold_impl_item_method(&mut self, mut i: syn::ImplItemMethod) -> syn::ImplItemMethod {
        let new_block_tokens = build_block(&i.sig.decl, &i.sig.ident, &i.block);
        let new_block = syn::parse2(new_block_tokens).unwrap();
        i.block = new_block;
        i
    }

    fn fold_item_fn(&mut self, mut i: syn::ItemFn) -> syn::ItemFn {
        let new_block_tokens = build_block(&*i.decl, &i.ident, &*i.block);
        let new_block = syn::parse2(new_block_tokens).unwrap();
        i.block = Box::new(new_block);
        i
    }
}
