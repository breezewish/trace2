use proc_macro2::Span;
use proc_macro2::TokenStream as TokenStream2;
use quote::ToTokens;
use syn;

pub struct Codegen;

impl Codegen {
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
                    Self::extract_printable_args(pat, extract_target);
                }
                for pat in tuple_struct.pat.back.iter() {
                    Self::extract_printable_args(pat, extract_target);
                }
            }
            syn::Pat::Tuple(ref tuple) => {
                for pat in tuple.front.iter() {
                    Self::extract_printable_args(pat, extract_target);
                }
                for pat in tuple.back.iter() {
                    Self::extract_printable_args(pat, extract_target);
                }
            }
            syn::Pat::Struct(ref structure) => {
                for pat in structure.fields.iter() {
                    Self::extract_printable_args(&*pat.pat, extract_target);
                }
            }
            syn::Pat::Slice(ref slice) => {
                for pat in slice.front.iter() {
                    Self::extract_printable_args(pat, extract_target);
                }
                if let Some(ref pat) = slice.middle {
                    Self::extract_printable_args(&*pat, extract_target);
                }
                for pat in slice.back.iter() {
                    Self::extract_printable_args(pat, extract_target);
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
    fn build_begin_trace_statement(fn_decl: &syn::FnDecl, fn_name: &str) -> TokenStream2 {
        let mut args = vec![];
        for fn_arg in fn_decl.inputs.iter() {
            match fn_arg {
                syn::FnArg::SelfRef(_) | syn::FnArg::SelfValue(_) => {
                    // ignore self arg
                }
                syn::FnArg::Captured(ref arg) => {
                    Self::extract_printable_args(&arg.pat, &mut args);
                }
                syn::FnArg::Inferred(ref arg_pat) => {
                    Self::extract_printable_args(arg_pat, &mut args);
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

        let format = format!("{{}} {{}}::{}({})", fn_name, format_args);

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
    fn build_end_trace_statement(fn_name: &str) -> TokenStream2 {
        let format = format!("{{}} {{}}::{} = {{:?}}", fn_name);

        quote! {
            trace!(#format, "<".repeat(__level * 4), module_path!(), __ret)
        }
    }

    /// Build the return type of the inner closure.
    ///
    /// We should provide type as much as possible to eliminate type inference failure.
    fn build_return_type(fn_decl: &syn::FnDecl) -> TokenStream2 {
        let ret_type = match &fn_decl.output {
            syn::ReturnType::Default => None,
            syn::ReturnType::Type(_, ref ret_type) => match **ret_type {
                // We don't write the type if the return type is impl trait.
                syn::Type::ImplTrait(_) => None,
                _ => Some(ret_type),
            },
        };
        match ret_type {
            None => syn::token::Underscore::new(Span::call_site()).into_token_stream(),
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
    pub fn build_block(decl: &syn::FnDecl, ident: &syn::Ident, impl_type: Option<&syn::Type>, block: &syn::Block) -> TokenStream2 {
        let fn_name = {
            let impl_type_str = match impl_type {
                None => "".to_owned(),
                Some(impl_type) => {
                    format!("{}::", quote!(#impl_type))
                }
            };
            format!("{}{}", impl_type_str, ident)
        };

        let begin_trace = Self::build_begin_trace_statement(decl, &fn_name);
        let end_trace = Self::build_end_trace_statement(&fn_name);
        let return_type = Self::build_return_type(decl);
        quote! {
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
}
