use proc_macro;
use quote::{quote, format_ident};
use syn::{self, parse_macro_input, DeriveInput};
use darling::FromDeriveInput;

use crate::parsing::{self, Input};

fn named_struct_deserialize_impl(struct_name: &syn::Ident, fields: &Vec<parsing::Field>) -> proc_macro2::TokenStream {
    let field_names = fields.iter().map(|field| field.ident.as_ref().unwrap());
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

fn unnamed_struct_deserialize_impl(struct_name: &syn::Ident, fields: &Vec<parsing::Field>) -> proc_macro2::TokenStream {
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
                Ok(Self(
                    #(<#field_withs as crate::de::Deserialize<'de>>::deserialize(input)?),*
                ))
            }
        }
    }).into()
}

fn unit_struct_deserialize_impl(struct_name: &syn::Ident) -> proc_macro2::TokenStream {
    (quote! {
        impl crate::de::Deserialize<'_> for #struct_name {
            type Value = Self;

            fn deserialize(input: &mut crate::de::ByteReader<'_>) -> crate::de::Result<Self> {
                Ok(Self)
            }
        }
    }).into()
}

fn packet_impl(input: &Input) -> proc_macro2::TokenStream {
    let struct_name = &input.ident;
    let id = input.attrs.id;
    (quote! {
        impl crate::Packet for #struct_name {
            const ID: i32 = #id;
        }
    }).into()
}

pub fn packet_derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let input = match Input::from_derive_input(&input) {
        Ok(inp) => inp,
        Err(err) => return err.write_errors().into(),
    };

    let struct_name = &input.ident;
    let deserialize_impl = match &input.data {
        parsing::DataStruct::Named(fields) => named_struct_deserialize_impl(struct_name, fields),
        parsing::DataStruct::Unnamed(fields) => unnamed_struct_deserialize_impl(struct_name, fields),
        parsing::DataStruct::Unit => unit_struct_deserialize_impl(struct_name),
    };
    let packet_impl = packet_impl(&input);
    (quote! {
        #deserialize_impl

        #packet_impl
    }).into()
}