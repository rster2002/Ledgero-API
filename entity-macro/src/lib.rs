mod col_mapping;
mod create_by_fn;

use crate::col_mapping::ColMapping;
use crate::create_by_fn::create_by_fn;
use convert_case::Case;
use proc_macro::TokenStream;
use quote::__private::TokenTree;
use quote::{quote, ToTokens};
use syn::{parse_macro_input, Attribute, Data, DeriveInput, Ident};

#[proc_macro_derive(Entity)]
pub fn derive(input: TokenStream) -> TokenStream {
    let DeriveInput {
        ident, data, attrs, ..
    } = parse_macro_input!(input as DeriveInput);

    let table_name = get_table_name(&attrs).unwrap_or_else(|| ident.to_string());

    let naming = get_rename_all_casing(&attrs).unwrap_or(Case::Pascal);

    let Data::Struct(data) = data else {
        panic!("Not a struct");
    };

    let columns: Vec<ColMapping<'_>> = data
        .fields
        .iter()
        .filter_map(|field| ColMapping::from_field(field, &naming))
        .collect();

    let mut value_parts = vec![];
    let mut i = 1;
    while i <= columns.len() {
        value_parts.push(format!("${}", i));
        i += 1;
    }

    let insert_statement = format!(
        "INSERT INTO {} VALUES ({})",
        table_name,
        value_parts.join(", ")
    );

    let col_idents: Vec<&Ident> = columns.iter().map(|c| c.source_ident).collect();

    let function_declarations: Vec<quote::__private::TokenStream> = columns
        .iter()
        .map(|c| create_by_fn(c, &columns, &table_name, &ident))
        .collect();

    let result = quote! {
        impl #ident {
            pub async fn create(&self, pool: &::sqlx::Pool<::sqlx::Postgres>) -> ::sqlx::Result<()> {
                sqlx::query!(
                        #insert_statement,
                        #(self.#col_idents),*
                    )
                    .execute(pool)
                    .await?;

                Ok(())
            }

            // #(#function_declarations)*
        }
    };

    result.into()
}

fn get_table_name(attributes: &[Attribute]) -> Option<String> {
    let table_name_attribute = attributes.iter().find(|attribute| {
        attribute
            .path
            .segments
            .iter()
            .any(|segment| segment.ident == *"table_name")
    });

    let Some(table_name_attribute) = table_name_attribute else {
        panic!("Incorrect table name attribute");
    };

    let mut stream = table_name_attribute.tokens.to_token_stream().into_iter();

    let Some(TokenTree::Group(group)) = stream.next() else {
        panic!("Incorrect table name attribute");
    };

    let Some(TokenTree::Literal(literal)) = group.stream().into_iter().next() else {
        panic!("Incorrect table name attribute");
    };

    Some(literal.to_string().replacen('\"', "", 2))
}

fn get_rename_all_casing(attributes: &[Attribute]) -> Option<Case> {
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

    if ident != "rename_all" {
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

    let literal_string = &*literal.to_string().replacen('\"', "", 2);

    let casing = match literal_string {
        "lowercase" => Case::Lower,
        "UPPERCASE" => Case::Upper,
        "camelCase" => Case::Camel,
        "SCREAMING_SNAKE_CASE" => Case::ScreamingSnake,
        "kebab-case" => Case::Kebab,
        "snake_case" => Case::Snake,
        _ => Case::Pascal,
    };

    Some(casing)
}

#[proc_macro_attribute]
pub fn table_name(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}
