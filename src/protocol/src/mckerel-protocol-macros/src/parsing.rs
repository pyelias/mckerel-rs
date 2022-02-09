use syn::{self, Type, Ident, DeriveInput};
use syn::spanned::Spanned;
use darling::{self, FromAttributes, FromField, FromDeriveInput};

#[derive(FromAttributes)]
#[darling(attributes(darling))]
pub struct FieldAttrs {
    #[darling(default)]
    pub with: Option<String>,
}

pub struct Field {
    pub ident: Option<Ident>,
    pub ty: Type,
    pub attrs: FieldAttrs,
}

// apparently darling's derive macros can't map on attrs?
impl FromField for Field {
    fn from_field(field: &syn::Field) -> darling::Result<Self> {
        Ok(Self {
            ident: field.ident.clone(),
            ty: field.ty.clone(),
            attrs: FieldAttrs::from_attributes(&field.attrs)?
        })
    }
}

pub enum DataStruct {
    Named(Vec<Field>),
    Unnamed(Vec<Field>),
    Unit,
}

impl DataStruct {
    fn new(data_struct: &syn::DataStruct) -> darling::Result<Self> {
        match &data_struct.fields {
            syn::Fields::Named(fields) => Ok(Self::Named(fields.named.iter().map(Field::from_field).collect::<darling::Result<_>>()?)),
            syn::Fields::Unnamed(fields) => Ok(Self::Unnamed(fields.unnamed.iter().map(Field::from_field).collect::<darling::Result<_>>()?)),
            syn::Fields::Unit => Ok(Self::Unit)
        }
    }

    fn from_data(data: &syn::Data) -> darling::Result<Self> {
        match data {
            syn::Data::Struct(data_struct) => Ok(Self::new(data_struct)?),
            syn::Data::Enum(data_enum) => Err(syn::Error::new(data_enum.enum_token.span(), "expected struct for packet derive").into()),
            syn::Data::Union(data_union) => Err(syn::Error::new(data_union.union_token.span(), "expected struct for packet derive").into())
        }
    }
}

#[derive(FromAttributes)]
#[darling(attributes(packet))]
pub struct StructAttrs {
    pub id: i32,
}

pub struct Input {
    pub ident: Ident,
    pub data: DataStruct,
    pub attrs: StructAttrs,
}

impl FromDeriveInput for Input {
    fn from_derive_input(input: &DeriveInput) -> darling::Result<Self> {
        Ok(Self {
            ident: input.ident.clone(),
            data: DataStruct::from_data(&input.data)?,
            attrs: StructAttrs::from_attributes(&input.attrs)?,
        })
    }
}