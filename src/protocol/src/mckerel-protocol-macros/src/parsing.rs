use syn::{self, Type, Result as SynResult, Ident, DeriveInput};
use syn::spanned::Spanned;

pub struct FieldAttrs {
    pub with: Option<String>,
}

fn merge_options<T>(a: &mut Option<T>, b: Option<T>) -> Result<(), ()> {
    if a.is_some() {
        if b.is_some() {
            return Err(());
        }
    } else {
        *a = b;
    };
    Ok(())
}

fn get_lit_str(lit: &syn::Lit) -> SynResult<String> {
    if let syn::Lit::Str(lit) = lit {
        Ok(lit.value())
    } else {
        Err(syn::Error::new(lit.span(), "cannot parse attribute; expected string literal"))
    }
}

impl FieldAttrs {
    fn new() -> Self {
        Self {
            with: None
        }
    }

    fn add_nested_meta(&mut self, meta: syn::NestedMeta) -> SynResult<()> {
        use syn::NestedMeta::*;
        use syn::Meta::*;
        match &meta {
            Meta(NameValue(nv)) if nv.path.is_ident("with") => {
                if let Err(_) = merge_options(&mut self.with, Some(get_lit_str(&nv.lit)?)) {
                    return Err(syn::Error::new(meta.span(), "duplicate with"))
                }
            },
            _ => return Err(syn::Error::new(meta.span(), "cannot parse attribute"))
        };
        Ok(())
    }

    fn add_meta(&mut self, meta: syn::Meta) -> SynResult<()> {
        if let syn::Meta::List(meta_list) = meta {
            if meta_list.path.is_ident("packet") {
                for nested in meta_list.nested.into_iter() {
                    self.add_nested_meta(nested)?;
                }
            }
        }
        Ok(())
    }
}

pub struct FieldNamed {
    pub ident: Ident,
    pub ty: Type,
    pub attrs: FieldAttrs,
}

impl FieldNamed {
    fn new(field: syn::Field) -> SynResult<Self> {
        let mut attrs = FieldAttrs::new();
        for attrs_line in field.attrs {
            attrs.add_meta(attrs_line.parse_meta()?)?;
        }

        Ok(Self {
            ident: field.ident.unwrap(), // guaranteed to exist if this function is being called
            ty: field.ty,
            attrs
        })
    }
}

pub struct FieldUnnamed {
    pub ty: Type,
    pub attrs: FieldAttrs,
}

impl FieldUnnamed {
    fn new(field: syn::Field) -> SynResult<Self> {
        let mut attrs = FieldAttrs::new();
        for attrs_line in field.attrs {
            attrs.add_meta(attrs_line.parse_meta()?)?;
        }

        Ok(Self {
            ty: field.ty,
            attrs
        })
    }
}

pub enum DataStruct {
    Named(Vec<FieldNamed>),
    Unnamed(Vec<FieldUnnamed>),
    Unit,
}

impl DataStruct {
    fn new(data_struct: syn::DataStruct) -> SynResult<Self> {
        match data_struct.fields {
            syn::Fields::Named(fields) => Ok(Self::Named(fields.named.into_iter().map(FieldNamed::new).collect::<SynResult<_>>()?)),
            syn::Fields::Unnamed(fields) => Ok(Self::Unnamed(fields.unnamed.into_iter().map(FieldUnnamed::new).collect::<SynResult<_>>()?)),
            syn::Fields::Unit => Ok(Self::Unit)
        }
    }

    fn from_data(data: syn::Data) -> SynResult<Self> {
        match data {
            syn::Data::Struct(data_struct) => Ok(Self::new(data_struct)?),
            syn::Data::Enum(data_enum) => Err(syn::Error::new(data_enum.enum_token.span(), "expected struct for packet derive")),
            syn::Data::Union(data_union) => Err(syn::Error::new(data_union.union_token.span(), "expected struct for packet derive"))
        }
    }
}

pub struct Input {
    pub ident: Ident,
    pub data: DataStruct,
}

impl Input {
    pub fn new(input: DeriveInput) -> SynResult<Self> {
        Ok(Self {
            ident: input.ident,
            data: DataStruct::from_data(input.data)?,
        })
    }
}