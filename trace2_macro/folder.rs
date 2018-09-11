use proc_macro2::TokenStream as TokenStream2;
use quote::ToTokens;
use syn;
use syn::fold::Fold;

use super::codegen::Codegen;
use super::config::Config;

/// The scope that fold begins, i.e. the location that `#[trace2]` is placed.
///
/// When we visiting a AST node marked as `#[trace2]` whose scope is smaller, the node
/// and its children will not be processed, because it will be invoked with a new folder
/// later by the compiler.
#[derive(PartialOrd, PartialEq, Debug)]
enum FoldScope {
    Fn = 0,
    Impl = 1,
    Mod = 2,
}

/// The AST visitor and modifier.
#[derive(Debug)]
pub struct Folder {
    /// The target scope when this folder is constructed.
    scope: FoldScope,

    /// The current impl block's type. It is set when entering a `impl` block,
    /// and unset when existing.
    current_impl: Option<syn::Type>,
}

impl Folder {
    fn new(scope: FoldScope) -> Self {
        Self {
            scope,
            current_impl: None,
        }
    }
}

impl Folder {
    pub fn fold(args: TokenStream2, input: TokenStream2) -> TokenStream2 {
        let config = syn::parse2::<Config>(args).expect("Failed to parse attribute configurations");
        if config.ignore {
            return input;
        }

        // Try to parse as `mod {}`
        let body = syn::parse2::<syn::ItemMod>(input.clone());
        if let Ok(body) = body {
            return Folder::new(FoldScope::Mod)
                .fold_item_mod(body)
                .into_token_stream();
        }

        // Try to parse as `fn()`
        let body = syn::parse2::<syn::ItemFn>(input.clone());
        if let Ok(body) = body {
            return Folder::new(FoldScope::Fn)
                .fold_item_fn(body)
                .into_token_stream();
        }

        // Try to parse as `impl {}`
        let body = syn::parse2::<syn::ItemImpl>(input.clone());
        if let Ok(body) = body {
            return Folder::new(FoldScope::Impl)
                .fold_item_impl(body)
                .into_token_stream();
        }

        panic!("Invalid attribute position, only supports function, impl and mod.");
    }

    /// Extract configuration of `#[trace2]` attribute from a list of attributes.
    ///
    /// Returns `None` if attribute is not specified in the list.
    fn extract_macro_config(attributes: &[syn::Attribute]) -> Option<Config> {
        // Path ends with `trace2` is considered to hit.
        // TODO: User may rename the macro. We cannot handle it here.
        for attr in attributes {
            let some_last = attr.path.segments.iter().last();
            if let Some(last) = some_last {
                if last.ident.to_string() == "trace2" {
                    // Try to parse the configuration
                    let config = syn::parse2::<AttrTTS>(attr.tts.clone())
                        .expect("Failed to parse attribute configurations");
                    return Some(config.0);
                }
            }
        }
        None
    }
}

impl Fold for Folder {
    fn fold_impl_item_method(&mut self, mut i: syn::ImplItemMethod) -> syn::ImplItemMethod {
        let some_config = Self::extract_macro_config(&i.attrs);
        if let Some(_) = some_config {
            // we are folding from a larger scope, ignore this
            if self.scope > FoldScope::Fn {
                return i;
            }
        }
        let new_block_tokens = Codegen::build_block(&i.sig.decl, &i.sig.ident, self.current_impl.as_ref(), &i.block);
        let new_block = syn::parse2(new_block_tokens).unwrap();
        i.block = new_block;
        i
    }

    fn fold_item_fn(&mut self, mut i: syn::ItemFn) -> syn::ItemFn {
        let some_config = Self::extract_macro_config(&i.attrs);
        if let Some(_) = some_config {
            // we are folding from a larger scope, ignore this
            if self.scope > FoldScope::Fn {
                return i;
            }
        }
        let new_block_tokens = Codegen::build_block(&*i.decl, &i.ident, None, &*i.block);
        let new_block = syn::parse2(new_block_tokens).unwrap();
        i.block = Box::new(new_block);
        i
    }

    fn fold_item_impl(&mut self, i: syn::ItemImpl) -> syn::ItemImpl {
        // Maybe called from either:
        // - Children traversal from a mod scope folder
        // - Root traversal of a impl scope folder
        //
        // For both cases, we record this impl block's type, so that when we further traverse
        // children `fn()`, we will know where it comes from.
        //
        // However, children `fn()` might be ignored and not traversed in this folder when it has
        // a `#[trace]` attribute. In this case, we attach meta data to that `#[trace]` attribute.
        // This will be done in `fold_impl_item_method`.

        // TODO: Rewrite children `fn()`'s attribute if it is going be ignored.

        let some_config = Self::extract_macro_config(&i.attrs);
        if let Some(_) = some_config {
            // we are folding from a larger scope, ignore this
            if self.scope > FoldScope::Impl {
                return i;
            }
        }

        // Extract current impl's type before traversing children. If its path has multiple parts,
        // only last part is preserved.
        self.current_impl = Some(TypeFolder.fold_type(*(i.self_ty).clone()));

        let ret = syn::fold::fold_item_impl(self, i);

        // Clear current impl's type when children traversal is completed.
        self.current_impl = None;
        ret
    }
}

/// Implements `syn::fold::Fold`. It changes a `syn::Type` by only preserving the last part of a
/// `syn::TypePath`. In this way, we will get a cleaner and simpler type to be printed out.
/// It should be invoked by `fold_type`.
struct TypeFolder;

impl Fold for TypeFolder {
    fn fold_path(&mut self, mut i: syn::Path) -> syn::Path {
        // When we meet a path, only preserve the last part.
        let segments_len = i.segments.len();
        if segments_len > 1 {
            i.segments = i.segments.into_iter().skip(segments_len - 1).collect();
        }
        i
    }
}


#[derive(Debug)]
struct AttrTTS(Config);

impl syn::synom::Synom for AttrTTS {
    named!(parse -> Self, do_parse!(
        config: option!(parens!(syn!(Config))) >>
        (AttrTTS(config.map(|pair| pair.1).unwrap_or_default()))
    ));
}

#[cfg(test)]
mod test {
    use super::AttrTTS;

    use syn;

    #[test]
    fn parse_attr_tts() {
        let attr_tts = syn::parse_str::<AttrTTS>("").unwrap();
        assert!(attr_tts.0.ignore, false);

        let attr_tts = syn::parse_str::<AttrTTS>("(ignore)").unwrap();
        assert_eq!(attr_tts.0.ignore, true);
    }
}
