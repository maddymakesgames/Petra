mod swizzle;
mod vertex;

use quote::ToTokens;
use syn::{parse_macro_input, DeriveInput};

use crate::{swizzle::SwizzleInput, vertex::gen_vertex};

extern crate proc_macro;

#[proc_macro]
pub fn swizzles(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as SwizzleInput);
    input.to_token_stream().into()
}

#[proc_macro_derive(Vertex, attributes(location, step_mode))]
pub fn vertex(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    gen_vertex(input)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}
