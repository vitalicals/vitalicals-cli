//! The derive for vital scripts

#![recursion_limit = "128"]
extern crate proc_macro;

#[macro_use]
extern crate syn;

#[macro_use]
extern crate quote;

mod decode_operand;
mod encode;
mod opcode;

use syn::{spanned::Spanned, Data, DeriveInput, Error, Field, Fields};

/// Derive
#[proc_macro_derive(BasicOpcode, attributes(codec))]
pub fn basic_opcode_derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let mut input: DeriveInput = match syn::parse(input) {
        Ok(input) => input,
        Err(e) => return e.to_compile_error().into(),
    };

    let generate = quote! {};

    generate.into()
}
