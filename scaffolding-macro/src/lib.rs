extern crate proc_macro;
extern crate proc_macro2;
extern crate quote;
extern crate syn;

use std::mem;

use proc_macro::TokenStream;
use quote::quote;
use syn::{
    parse::{Parse, Parser},
    spanned::Spanned,
};

#[proc_macro_attribute]
pub fn scaffold(_args: TokenStream, input: TokenStream) -> TokenStream {
    let output = match syn::Item::parse.parse(input.clone()) {
        Ok(syn::Item::Fn(mut item_fn)) => {
            let mut errors = Vec::new();

            if !item_fn.sig.inputs.is_empty() {
                errors.push(syn::parse::Error::new(
                    item_fn.sig.span(),
                    "unsupported function with arguments",
                ));
            }

            if errors.is_empty() {
                let attrs = mem::take(&mut item_fn.attrs);
                let name = &item_fn.sig.ident;

                quote! {
                    #[test]
                    #(#attrs)*
                    fn #name() {
                        #item_fn
                       ::scaffolding::scaffold(#name())
                    }
                }
            } else {
                errors
                    .iter()
                    .map(syn::parse::Error::to_compile_error)
                    .collect()
            }
        }
        _ => {
            let span = proc_macro2::TokenStream::from(input).span();
            let msg = "#[scaffold] is only supported on functions";

            syn::parse::Error::new(span, msg).to_compile_error()
        }
    };

    output.into()
}
