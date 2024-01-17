//! The derive for vital scripts

#![recursion_limit = "128"]
extern crate proc_macro;

extern crate syn;

#[macro_use]
extern crate quote;

mod decode_operand;
mod encode;

use syn::DeriveInput;

/// Derive For implementation basic opcode
#[proc_macro_derive(BasicOpcode, attributes(codec))]
pub fn basic_opcode_derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input: DeriveInput = match syn::parse(input) {
        Ok(input) => input,
        Err(e) => return e.to_compile_error().into(),
    };

    let name = &input.ident;

    let generate = quote! {
        const _: () = {
            impl crate::op_basic::BasicOpcodeBase for #name {
                const ID: u8 = crate::opcodes::BasicOp::#name as u8;
            }
        };
    };

    generate.into()
}

/// Derive For implementation extension opcode
#[proc_macro_derive(ExtensionOpcode, attributes(codec))]
pub fn extension_opcode_derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input: DeriveInput = match syn::parse(input) {
        Ok(input) => input,
        Err(e) => return e.to_compile_error().into(),
    };

    let name = &input.ident;

    let generate = quote! {
        const _: () = {
            impl crate::op_extension::ExtensionOpcodeBase for #name {
                const ID: u16 = crate::opcodes::ExtensionOp::#name as u16;
            }
        };
    };

    generate.into()
}
