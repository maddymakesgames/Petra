use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{Data, DeriveInput, Error, Fields, Lit, LitInt, Result};

pub fn gen_vertex(input: DeriveInput) -> Result<TokenStream> {
    if input.generics.lt_token.is_some() {
        return Err(Error::new(
            Span::call_site(),
            "Cannot implement Vertex for a generic type",
        ));
    }
    let name = input.ident;

    let mut fields_data = Vec::new();

    match input.data {
        Data::Struct(s) => match s.fields {
            Fields::Named(fields) =>
                for field in &fields.named {
                    fields_data.push((field.ty.clone(), field.attrs.clone()));
                },
            Fields::Unnamed(fields) =>
                for field in &fields.unnamed {
                    fields_data.push((field.ty.clone(), field.attrs.clone()))
                },
            Fields::Unit =>
                return Err(Error::new(
                    Span::call_site(),
                    "Cannot implement Vertex for a unit struct",
                )),
        },
        Data::Enum(_e) =>
            return Err(Error::new(
                Span::call_site(),
                "Cannot implement Vertex for an enum",
            )),
        Data::Union(_u) =>
            return Err(Error::new(
                Span::call_site(),
                "Cannot implement Vertex for a union",
            )),
    };

    let formats = fields_data.iter().map(|(f, _)| {
        quote! {
            <#f as ::render_lib::vertex::VertexField>::FORMAT
        }
    });

    let mut offsets = vec![quote!(0_u64)];
    let mut locations = Vec::new();
    let mut i = 0;

    for (kind, attrs) in &fields_data {
        let prev_offset = offsets.last().unwrap();
        let mut found = false;
        for attr in attrs {
            match attr.parse_meta()? {
                syn::Meta::NameValue(nv) =>
                    if nv.path.is_ident("location") {
                        if let Lit::Int(_) = nv.lit {
                            locations.push(nv.lit.clone());
                        } else {
                            return Err(Error::new(
                                nv.lit.span(),
                                "Location attributes must specify an int literal for the location",
                            ));
                        }
                        found = true;
                        break;
                    },
                _ => continue,
            }
        }

        if !found {
            locations.push(Lit::Int(LitInt::new(&i.to_string(), Span::call_site())));
            i += 1;
        }

        offsets.push(quote! {
            #prev_offset + std::mem::size_of::<#kind>() as u64
        });
    }

    Ok(quote! {
        impl ::render_lib::vertex::Vertex for #name {
            const FIELDS: &'static [::render_lib::vertex::VertexAttribute] = &[
                #(::render_lib::vertex::VertexAttribute {
                    format: #formats,
                    offset: #offsets,
                    shader_location: #locations as u32
                }),*
            ];

            const STEP_MODE: ::render_lib::vertex::VertexStepMode = ::render_lib::vertex::VertexStepMode::Vertex;
        }
    })
}
