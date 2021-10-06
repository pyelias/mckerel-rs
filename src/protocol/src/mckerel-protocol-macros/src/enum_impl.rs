use proc_macro;
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::{self, braced, parse_macro_input, Ident, LitInt, Token};

struct EnumField {
    name: Ident,
    tag: LitInt,
}

impl Parse for EnumField {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let name = input.parse()?;
        input.parse::<Token![=]>()?;
        let tag = input.parse()?;
        Ok(Self { name, tag })
    }
}

struct EnumInput {
    tag_type: Ident,
    name: Ident,
    fields: Vec<EnumField>,
}

impl Parse for EnumInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let tag_type = input.parse()?;
        let name = input.parse()?;
        let fields;
        braced!(fields in input);
        let fields = fields.parse_terminated::<_, Token![,]>(Parse::parse)?;
        let fields = fields.into_iter().collect();
        Ok(Self {
            tag_type,
            name,
            fields,
        })
    }
}

pub fn enum_impl(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as EnumInput);

    let tag_type = input.tag_type;
    let enum_name = input.name;
    let tag_vals: Vec<&LitInt> = input.fields.iter().map(|f| &f.tag).collect();
    let field_names: Vec<&Ident> = input.fields.iter().map(|f| &f.name).collect();

    (quote! {
        pub enum #enum_name {
            #(#field_names),*
        }

        impl crate::de::Deserialize<'_> for #enum_name {
            type Value = Self;

            fn deserialize(input: &mut crate::de::ByteReader<'_>) -> crate::de::Result<Self> {
                let tag = <#tag_type as crate::de::Deserialize>::deserialize(input)?;
                match tag {
                    #(#tag_vals => Ok(Self::#field_names),)*
                    default => Err(crate::de::Error::BadEnumTag)
                }
            }
        }
    }).into()
}
