use proc_macro::{Span};
use convert_case::{Case, Casing};
use quote::__private::TokenStream;
use syn::{Ident};
use quote::{quote, ToTokens};
use crate::col_mapping::ColMapping;

pub fn create_by_fn(col_mapping: &ColMapping, mappings: &[ColMapping], table_name: &String, struct_ident: &Ident) -> TokenStream {
    let name = format!("by_{}", col_mapping.source_ident);
    let function_ident = Ident::new(&name, Span::call_site().into());
    let value_type = col_mapping.argument_type;

    let mut type_string = value_type.to_token_stream().to_string();

    if type_string == *"String" {
        type_string = String::from("impl Into<String>");
    }

    let type_string: TokenStream = type_string.parse().unwrap();

    let select_query = format!("SELECT * FROM {} WHERE {} = $1", table_name, col_mapping.col_name);

    let bindings = mappings.iter()
        .map(|col| {
            let target = col.source_ident;
            let record_binding: TokenStream = format!("result.{}", col.col_name.to_case(Case::Flat)).parse().unwrap();

            quote! { #target: #record_binding }
        });

    quote! {
        pub async fn #function_ident(pool: &::sqlx::Pool<::sqlx::Postgres>, value: #type_string) -> ::sqlx::Result<Option<Self>> {
            let db_result = sqlx::query!(
                    #select_query,
                    value.into()
                )
                .fetch_optional(pool)
                .await?;

            let Some(result) = db_result else {
                return Ok(None);
            };

            Ok(Some( #struct_ident {
                #(#bindings),*
            }))
        }
    }
}
