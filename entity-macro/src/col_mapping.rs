use convert_case::{Case, Casing};
use quote::ToTokens;
use quote::__private::TokenTree;
use syn::{Attribute, Field, Ident, Type};

pub struct ColMapping<'a> {
    pub(crate) source_ident: &'a Ident,
    pub(crate) col_name: String,
    pub(crate) argument_type: &'a Type,
}

impl<'a> ColMapping<'a> {
    pub fn from_field(field: &'a Field, casing: &Case) -> Option<Self> {
        let Some(ident) = &field.ident else {
            return None;
        };

        let col_name = if let Some(rename) = get_rename_attribute(&field.attrs) {
            rename
        } else {
            ident.to_string().to_case(*casing)
        };

        dbg!(&col_name);

        Some(Self {
            source_ident: ident,
            argument_type: &field.ty,
            col_name,
        })
    }
}

fn get_rename_attribute(attributes: &[Attribute]) -> Option<String> {
    let sqlx_attribute = attributes.iter().find(|attribute| {
        attribute
            .path
            .segments
            .iter()
            .any(|segment| segment.ident == *"sqlx")
    });

    let Some(sqlx_attribute) = sqlx_attribute else {
        return None;
    };

    let mut stream = sqlx_attribute.tokens.to_token_stream().into_iter();

    let Some(TokenTree::Group(group)) = stream.next() else {
        return None;
    };

    let mut group_stream = group.stream().into_iter();

    let Some(TokenTree::Ident(ident)) = group_stream.next() else {
        return None;
    };

    if ident != "rename" {
        return None;
    }

    let Some(TokenTree::Punct(punct)) = group_stream.next() else {
        return None;
    };

    if punct.to_string() != "=" {
        return None;
    }

    let Some(TokenTree::Literal(literal)) = group_stream.next() else {
        return None;
    };

    Some(literal.to_string().replacen('\"', "", 2))
}
