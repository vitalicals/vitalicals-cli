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
    let input: DeriveInput = match syn::parse(input) {
        Ok(input) => input,
        Err(e) => return e.to_compile_error().into(),
    };

    let name = &input.ident;

    let generate = quote! {
        const _: () = {
            impl crate::basic::Opcode for #name {
                const ID: u8 = crate::opcodes::BasicOp::#name as u8;
            }
        };
    };

    generate.into()
}
