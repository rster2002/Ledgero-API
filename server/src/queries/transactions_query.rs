use rocket::time::format_description::well_known::Rfc3339;
use sqlx::{Postgres, QueryBuilder};

use crate::models::dto::bank_accounts::slim_bank_account_dto::SlimBankAccountDto;
use crate::models::dto::categories::slim_category_dto::SlimCategoryDto;
use crate::models::dto::categories::subcategories::slim_subcategory_dto::SlimSubcategoryDto;
use crate::models::dto::external_accounts::external_account_dto::ExternalAccountDto;
use crate::models::dto::pagination::pagination_query_dto::PaginationQueryDto;
use crate::models::dto::transactions::transaction_dto::TransactionDto;
use crate::models::entities::transaction::transaction_type::TransactionType;
use crate::prelude::*;
use crate::queries::transactions_query::transaction_record::TransactionRecord;
use crate::shared::DbPool;

mod transaction_record;

pub struct TransactionQuery<'a> {
    builder: QueryBuilder<'a, Postgres>,
}

impl<'a> TransactionQuery<'a> {
    pub fn new(user_id: impl Into<String>) -> Self {
        let mut builder = QueryBuilder::new(
            r#"
                SELECT
                    transactions.id as transaction_id, transaction_type, follow_number, original_description, transactions.description, date, complete_amount, amount, external_account_name, related_move_transaction,
                    c.Id as "category_id?", c.name as "category_name?", c.description as "category_description?", c.hex_color as "category_hex_color?",
                    s.Id as "subcategory_id?", s.name as "subcategory_name?", s.description as "subcategory_description?", s.hex_color as "subcategory_hex_color?",
                    b.Id as "bank_account_id?", b.iban as "bank_account_iban?", b.name as "bank_account_name?", b.description as "bank_account_description?", b.hex_color as "bank_account_hex_color?",
                    e.Id as "external_account_id?", e.name as "external_account_entity_name?", e.description as "external_account_description?", e.hex_color as "external_account_hex_color?", e.default_category_id as "external_account_default_category_id?", e.default_subcategory_id as "external_account_default_subcategory_id?"
                FROM transactions
                LEFT JOIN categories c on transactions.category_id = c.id
                LEFT JOIN subcategories s on transactions.subcategory_id = s.id
                LEFT JOIN bank_accounts b on transactions.bank_account_id = b.id
                LEFT JOIN external_accounts e on transactions.external_account_id = e.id
                WHERE transactions.user_id =
            "#,
        );

        builder.push_bind(user_id.into());

        Self { builder }
    }

    pub fn where_id(mut self, transaction_id: impl Into<String>) -> Self {
        self.builder.push(" AND transactions.id = ");
        self.builder.push_bind(transaction_id.into());
        self
    }

    pub fn where_category(mut self, category_id: impl Into<String>) -> Self {
        self.builder.push(" AND transactions.category_id = ");
        self.builder.push_bind(category_id.into());
        self
    }

    pub fn where_subcategory(mut self, subcategory_id: impl Into<String>) -> Self {
        self.builder.push(" AND transactions.subcategory_id = ");
        self.builder.push_bind(subcategory_id.into());
        self
    }

    pub fn where_type(mut self, transaction_type: TransactionType) -> Self {
        self.builder.push(" AND transaction_type = ");
        self.builder.push_bind::<&str>(transaction_type.into());
        self
    }

    pub fn where_type_not(mut self, transaction_type: TransactionType) -> Self {
        self.builder.push(" AND transaction_type != ");
        self.builder.push_bind::<&str>(transaction_type.into());
        self
    }

    pub fn where_external_account(mut self, external_account_id: impl Into<String>) -> Self {
        self.builder.push(" AND external_account_id = ");
        self.builder.push_bind(external_account_id.into());
        self
    }

    pub fn where_bank_account(mut self, bank_account_id: impl Into<String>) -> Self {
        self.builder.push(" AND bank_account_id = ");
        self.builder.push_bind(bank_account_id.into());
        self
    }

    pub fn paginate(mut self, pagination: &PaginationQueryDto) -> Self {
        self.builder.push(" OFFSET ");
        self.builder.push_bind(pagination.get_offset());
        self.builder.push(" LIMIT ");
        self.builder.push_bind(pagination.get_limit());
        self
    }

    pub fn order(mut self) -> Self {
        self.builder
            .push(" ORDER BY date DESC, order_indicator DESC ");
        self
    }

    pub async fn fetch_one(mut self, pool: &DbPool) -> Result<TransactionDto> {
        let record = self.builder.build_query_as().fetch_one(pool).await?;

        Ok(TransactionQuery::map_record(record))
    }

    pub async fn fetch_all(mut self, pool: &DbPool) -> Result<Vec<TransactionDto>> {
        let records = self.builder.build_query_as().fetch_all(pool).await?;

        let transactions = records
            .into_iter()
            .map(TransactionQuery::map_record)
            .collect();

        Ok(transactions)
    }

    fn map_record(record: TransactionRecord) -> TransactionDto {
        let mut transaction = TransactionDto {
            id: record.transaction_id,
            transaction_type: TransactionType::from(&*record.transaction_type),
            follow_number: record.follow_number,
            original_description: record.original_description,
            description: record.description,
            complete_amount: record.complete_amount,
            amount: record.amount,
            date: record.date.format(&Rfc3339).expect("Incorrect formatting"),
            bank_account: None,
            category: None,
            subcategory: None,
            external_account_name: record.external_account_name,
            external_account: None,
            related_move_transaction: record.related_move_transaction,
        };

        if let Some(id) = record.bank_account_id {
            transaction.bank_account = Some(SlimBankAccountDto {
                id,
                iban: record
                    .bank_account_iban
                    .expect("Bank account id was not null, but bank account iban was"),
                name: record
                    .bank_account_name
                    .expect("Bank account id was not null, but bank account name was"),
                description: record
                    .bank_account_description
                    .expect("Bank account id was not null, but bank account description was"),
                hex_color: record
                    .bank_account_hex_color
                    .expect("Bank account id was not null, but bank account hex color was"),
            });
        }

        if let Some(id) = record.category_id {
            transaction.category = Some(SlimCategoryDto {
                id,
                name: record
                    .category_name
                    .expect("Category id was not null, but the category name was"),
                description: record
                    .category_description
                    .expect("Category id was not null, but the category description was"),
                hex_color: record
                    .category_hex_color
                    .expect("Category id was not null, but the category hex color was"),
            });
        }

        if let Some(id) = record.subcategory_id {
            transaction.subcategory = Some(SlimSubcategoryDto {
                id,
                name: record
                    .subcategory_name
                    .expect("Subcategory id was not null, but the subcategory name was"),
                description: record
                    .subcategory_description
                    .expect("Subcategory id was not null, but the subcategory description was"),
                hex_color: record
                    .subcategory_hex_color
                    .expect("Subcategory id was not null, but the subcategory hex color was"),
            });
        }

        if let Some(id) = record.external_account_id {
            transaction.external_account = Some(ExternalAccountDto {
                id,
                name: record
                    .external_account_entity_name
                    .expect("External account id was not null, but the external account name was"),
                description: record.external_account_description.expect(
                    "External account id was not null, but the external account description was",
                ),
                hex_color: record.external_account_hex_color.expect(
                    "External account id was not null, but the external account hex color was",
                ),
                default_category_id: record.external_account_default_category_id,
                default_subcategory_id: record.external_account_default_subcategory_id,
            })
        }

        transaction
    }
}
