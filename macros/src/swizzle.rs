use itertools::Itertools;
use proc_macro2::Ident;
use quote::{format_ident, quote, ToTokens, TokenStreamExt};
use syn::{
    bracketed,
    parenthesized,
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    Result,
    Token,
};

pub struct SwizzleInput {
    struct_name: Ident,
    to_types: Punctuated<Ident, Token![,]>,
    aliases: Punctuated<AliasGroup, Token![,]>,
}

impl Parse for SwizzleInput {
    fn parse(input: ParseStream) -> Result<Self> {
        let struct_name = input.parse()?;
        let mut bracketed;
        bracketed!(bracketed in input);
        let to_types = bracketed.parse_terminated(Ident::parse)?;
        bracketed!(bracketed in input);
        let aliases = bracketed.parse_terminated(AliasGroup::parse)?;

        Ok(SwizzleInput {
            struct_name,
            to_types,
            aliases,
        })
    }
}

impl ToTokens for SwizzleInput {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let aliases = self
            .aliases
            .iter()
            .map(|a| &a.0)
            .cloned()
            .collect::<Vec<_>>();
        let mut functions = Vec::new();

        for (i, to_type) in self.to_types.iter().enumerate() {
            for group in &aliases {
                let mut indecies = (0 .. group.len()).collect::<Vec<_>>();
                indecies = indecies.repeat(i + 2);
                for perm in indecies.iter().permutations(i + 2).unique() {
                    let a = perm
                        .iter()
                        .map(|v| group[**v].clone())
                        .collect::<Vec<_>>();
                    let func_name = a.iter().map(|v| v.to_string()).collect::<String>();
                    let func_name = format_ident!("{func_name}");
    
                    functions.push(quote! {
                        #[doc(hidden)]
                        pub fn #func_name(&self) -> #to_type {
                            #to_type::new(#(self.#a()),*)
                        }
                    });
                }
            }
        }

        let name = &self.struct_name;
        tokens.append_all(quote! {
            impl #name {
                #(#functions)*
            }
        })
    }
}

pub struct AliasGroup(Punctuated<Ident, Token![,]>);
impl Parse for AliasGroup {
    fn parse(input: ParseStream) -> Result<Self> {
        let braced;
        parenthesized!(braced in input);
        Ok(Self(braced.parse_terminated(Ident::parse)?))
    }
}
