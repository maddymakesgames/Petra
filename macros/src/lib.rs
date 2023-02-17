mod swizzle;

use quote::ToTokens;
use syn::parse_macro_input;

use crate::swizzle::SwizzleInput;

extern crate proc_macro;

#[proc_macro]
pub fn swizzles(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as SwizzleInput);
    input.to_token_stream().into()
}
