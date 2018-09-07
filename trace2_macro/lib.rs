#![recursion_limit = "128"]

extern crate proc_macro;
extern crate proc_macro2;
extern crate syn;
#[macro_use]
extern crate quote;

use proc_macro::TokenStream;

use quote::ToTokens;
use syn::fold::Fold;
use syn::spanned::Spanned;

#[proc_macro_attribute]
pub fn trace2(_args: TokenStream, input: TokenStream) -> TokenStream {
    fold(input).into()
}

fn fold(input: TokenStream) -> proc_macro2::TokenStream {
    let mut folder = Folder;
    // Try to parse as mod {}
    let body: Result<syn::ItemMod, _> = syn::parse(input.clone());
    if let Ok(body) = body {
        return folder.fold_item_mod(body).into_token_stream();
    }

    // Try to parse as fn()
    let body: Result<syn::ItemFn, _> = syn::parse(input.clone());
    if let Ok(body) = body {
        return folder.fold_item_fn(body).into_token_stream();
    }

    // Try to parse as impl {}
    let body: Result<syn::ItemImpl, _> = syn::parse(input.clone());
    if let Ok(body) = body {
        return folder.fold_item_impl(body).into_token_stream();
    }

    panic!("Invalid attribute position, only supports function, impl and mod.");
}

fn extract_printable_args<'a>(pat: &'a syn::Pat, extract_target: &mut Vec<&'a syn::Ident>) {
    match pat {
        syn::Pat::Wild(_) => {
            // ignore args without a name
        }
        syn::Pat::Path(_)
        | syn::Pat::Box(_)
        | syn::Pat::Ref(_)
        | syn::Pat::Lit(_)
        | syn::Pat::Range(_)
        | syn::Pat::Verbatim(_)
        | syn::Pat::Macro(_) => panic!("Unexpected argument pattern: {:?}", pat),
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

/// Build begin trace statement.
///
/// Output sample:
///
/// ```ignore
/// trace!("{} {}::foo(arg1: {:?}, arg2: {:?})", ">".repeat(..), module_path!(), arg1, arg2);
/// ```
fn build_begin_trace_statement(
    fn_decl: &syn::FnDecl,
    fn_ident: &syn::Ident,
) -> proc_macro2::TokenStream {
    let mut args = vec![];
    for fn_arg in fn_decl.inputs.iter() {
        match fn_arg {
            syn::FnArg::SelfRef(_) | syn::FnArg::SelfValue(_) => {
                // ignore self arg
            }
            syn::FnArg::Captured(ref arg) => {
                extract_printable_args(&arg.pat, &mut args);
            }
            syn::FnArg::Inferred(ref arg_pat) => {
                extract_printable_args(arg_pat, &mut args);
            }
            syn::FnArg::Ignored(_) => {
                // ignore ignored arg
            }
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

/// Build end trace statement.
///
/// Output sample:
///
/// ```ignore
/// trace!("{} {}::foo = {:?}", "<".repeat(..), module_path!(), __ret);
/// ```
fn build_end_trace_statement(fn_ident: &syn::Ident) -> proc_macro2::TokenStream {
    let format = format!("{{}} {{}}::{} = {{:?}}", fn_ident);

    quote! {
        trace!(#format, "<".repeat(__level * 4), module_path!(), __ret)
    }
}

/// Build the return type of the inner closure.
///
/// We should provide type as much as possible to eliminate type inference failure.
fn build_return_type(fn_decl: &syn::FnDecl) -> proc_macro2::TokenStream {
    let ret_type = match &fn_decl.output {
        syn::ReturnType::Default => None,
        syn::ReturnType::Type(_, ref ret_type) => match **ret_type {
            // We don't write the type if the return type is impl trait.
            syn::Type::ImplTrait(_) => None,
            _ => Some(ret_type),
        },
    };
    match ret_type {
        None => syn::token::Underscore::new(proc_macro2::Span::call_site()).into_token_stream(),
        Some(t) => t.clone().into_token_stream(),
    }
}

/// Transform and build a function block.
///
/// Suppose we receive:
/// ```ignore
/// (pub) fn foo<T>(arg1: T, arg2: foo) -> bool where T: bar {
///     ...
/// }
/// ```
///
/// This function will transform it into:
/// ```ignore
/// (pub) fn foo<T>(&self, arg1: T, arg2: foo) -> bool where T: bar {
///     trace!("{} foo(arg1: {:?}, arg2: {:?})", ">".repeat(..), arg1, arg2);
///     let mut __inner = move || {
///         let __inner_ret: bool = {
///             ...
///         };
///         #[allow(unreachable_code)]
///         __inner_ret
///     };
///     let __ret = __inner();
///     trace!("{} foo = {:?}", ">".repeat(..), __ret);
///     __ret
/// }
/// ```
fn build_block(
    decl: &syn::FnDecl,
    ident: &syn::Ident,
    block: &syn::Block,
) -> proc_macro2::TokenStream {
    let span = block.span();
    let begin_trace = build_begin_trace_statement(decl, ident);
    let end_trace = build_end_trace_statement(ident);
    let return_type = build_return_type(decl);
    quote_spanned! {span=>
        {
            use trace2;
            trace2::FUNC_CALL_LEVEL.with(|level| {
                let mut __level = level.get();
                __level = __level.saturating_add(1);
                level.set(__level);
                #begin_trace;
            });
            let mut __inner = move || {
                // Explicitly give types, so that Box<..> can be correctly inferred.
                let __inner_ret: #return_type = #block;

                #[allow(unreachable_code)]
                // This line might be unreachable, mute the warning. See unreachable test.
                __inner_ret
            };
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
