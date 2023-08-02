use std::collections::BTreeMap;

use sqlx::{Postgres, QueryBuilder};

use crate::models::dto::categories::category_dto::CategoryDto;
use crate::models::dto::categories::subcategories::subcategory_dto::SubcategoryDto;
use crate::prelude::*;
use crate::queries::categories_query::category_record::CategoryRecord;
use crate::shared::DbPool;

mod category_record;

pub struct CategoriesQuery<'a> {
    builder: QueryBuilder<'a, Postgres>,
}

impl<'a> CategoriesQuery<'a> {
    pub fn new(user_id: impl Into<String>) -> Self {
        let mut builder = QueryBuilder::new(
            r#"
                SELECT
                    categories.id as category_id, categories.name as category_name, categories.description as category_description, categories.hex_color as category_hex_color,
                    s.id as subcategory_id, s.name as subcategory_name, s.description as subcategory_description, s.hex_color as subcategory_hex_color,
                (
                    SELECT SUM(amount)::bigint
                    FROM transactions
                    WHERE categories.id = transactions.category_id
                ) AS amount,
                (
                    SELECT SUM(amount)::bigint
                    FROM transactions
                    WHERE categories.id = transactions.category_id AND s.Id = transactions.subcategory_id
                ) AS subcategory_amount
                FROM categories
                lEFT JOIN subcategories s ON categories.id = s.parent_category
                WHERE categories.user_id =
            "#,
        );

        builder.push_bind(user_id.into());

        Self { builder }
    }

    pub fn where_id(mut self, category_id: impl Into<String>) -> Self {
        self.builder.push(" AND categories.id = ");
        self.builder.push_bind(category_id.into());
        self
    }

    pub fn order(mut self) -> Self {
        self.builder.push(" ORDER BY order_index ASC");
        self
    }

    pub async fn fetch_one(self, pool: &DbPool) -> Result<CategoryDto> {
        let categories = self.fetch_all(pool).await?;

        let Some(record) = categories.into_iter().next() else {
            return Err(sqlx::Error::RowNotFound.into());
        };

        Ok(record)
    }

    pub async fn fetch_all(mut self, pool: &DbPool) -> Result<Vec<CategoryDto>> {
        let records = self.builder.build_query_as().fetch_all(pool).await?;

        CategoriesQuery::map_records(records)
    }

    fn map_records(records: Vec<CategoryRecord>) -> Result<Vec<CategoryDto>> {
        let mut category_map = BTreeMap::new();

        let ordering: Vec<String> = records.iter().map(|i| i.category_id.to_string()).collect();

        for record in records {
            if !category_map.contains_key(&record.category_id) {
                category_map.insert(
                    record.category_id.to_string(),
                    CategoryDto {
                        id: record.category_id.to_string(),
                        name: record.category_name,
                        description: record.category_description,
                        hex_color: record.category_hex_color,
                        amount: record.amount.unwrap_or(0),
                        subcategories: vec![],
                    },
                );
            }

            if let Some(id) = record.subcategory_id {
                let base_category = category_map
                    .get_mut(&record.category_id)
                    .expect("Base category should have been created");

                base_category.subcategories.push(SubcategoryDto {
                    id,
                    name: record
                        .subcategory_name
                        .expect("Subcategory id is set, but the name is not"),
                    description: record
                        .subcategory_description
                        .expect("Subcategory id is set, but the description is not"),
                    hex_color: record
                        .subcategory_hex_color
                        .expect("Subcategory id is set, but the hex color is not"),
                    amount: record.subcategory_amount.unwrap_or(0),
                });
            }
        }

        let categories = ordering
            .into_iter()
            .filter_map(|id| {
                if category_map.contains_key(&id) {
                    Some(category_map.remove(&*id).expect("Id not in map"))
                } else {
                    None
                }
            })
            .collect();

        Ok(categories)
    }
}
