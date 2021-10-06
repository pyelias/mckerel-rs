use proc_macro;
use quote::{quote, format_ident};
use syn::{self, parse_macro_input, DeriveInput};

use crate::parsing::{self, Input};

pub fn packet_derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let input = match Input::new(input) {
        Ok(inp) => inp,
        Err(err) => return err.to_compile_error().into(),
    };

    let struct_name = input.ident;
    let fields = match input.data {
        parsing::DataStruct::Named(fields) => fields,
        _ => todo!()
    };
    let field_names = fields.iter().map(|field| &field.ident);
    let field_withs = fields.iter().map(|field| {
        match &field.attrs.with {
            Some(ty) => {
                let ty = format_ident!("{}", ty);
                quote! { #ty }
            },
            None => {
                let ty = &field.ty;
                quote! { #ty }
            },
        }
    });

    (quote! {
        impl<'de> crate::de::Deserialize<'de> for #struct_name {
            type Value = Self;

            fn deserialize(input: &mut crate::de::ByteReader<'de>) -> crate::de::Result<Self> {
                Ok(Self {
                    #(#field_names: <#field_withs as crate::de::Deserialize<'de>>::deserialize(input)?),*
                })
            }
        }
    }).into()
}