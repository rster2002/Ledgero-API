mod transaction_record;

use rocket::time::format_description::well_known::Rfc3339;
use sqlx::{Postgres, QueryBuilder};

use crate::models::dto::bank_accounts::bank_account_dto::BankAccountDto;
use crate::models::dto::categories::slim_category_dto::SlimCategoryDto;
use crate::models::dto::categories::subcategories::slim_subcategory_dto::SlimSubcategoryDto;
use crate::models::dto::external_accounts::external_account_dto::ExternalAccountDto;
use crate::models::dto::pagination::pagination_query_dto::PaginationQueryDto;
use crate::models::dto::transactions::transaction_dto::TransactionDto;
use crate::models::entities::transaction::transaction_type::TransactionType;
use crate::prelude::*;
use crate::queries::transactions_query::transaction_record::TransactionRecord;
use crate::shared_types::DbPool;

pub struct TransactionQuery<'a> {
    builder: QueryBuilder<'a, Postgres>,
}

impl<'a> TransactionQuery<'a> {
    pub fn new(user_id: impl Into<String>) -> Self {
        let mut builder = QueryBuilder::new(
            r#"
                SELECT
                    transactions.Id as TransactionId, TransactionType, FollowNumber, OriginalDescription, transactions.Description, Date, CompleteAmount, Amount, ExternalAccountName,
                    c.Id as "CategoryId?", c.Name as "CategoryName?", c.Description as "CategoryDescription?", c.HexColor as "CategoryHexColor?",
                    s.Id as "SubcategoryId?", s.Name as "SubcategoryName?", s.Description as "SubcategoryDescription?", s.HexColor as "SubcategoryHexColor?",
                    b.Id as BankAccountId, b.Iban as BankAccountIban, b.Name as BankAccountName, b.Description as BankAccountDescription, b.HexColor as BankAccountHexColor,
                    e.Id as "ExternalAccountId?", e.Name as "ExternalAccountEntityName?", e.Description as "ExternalAccountDescription?", e.DefaultCategoryId as "ExternalAccounDefaultCategoryId?"
                FROM Transactions
                LEFT JOIN categories c on transactions.categoryid = c.id
                LEFT JOIN subcategories s on transactions.subcategoryid = s.id
                LEFT JOIN bankaccounts b on transactions.bankaccountid = b.id
                LEFT JOIN externalaccounts e on c.id = e.defaultcategoryid
                WHERE Transactions.UserId =
            "#,
        );

        builder.push_bind(user_id.into());

        Self { builder }
    }

    pub fn where_id(mut self, transaction_id: impl Into<String>) -> Self {
        self.builder.push(" AND Transactions.Id = ");
        self.builder.push_bind(transaction_id.into());
        self
    }

    pub fn where_category(mut self, category_id: impl Into<String>) -> Self {
        self.builder.push(" AND Transactions.CategoryId = ");
        self.builder.push_bind(category_id.into());
        self
    }

    pub fn where_subcategory(mut self, subcategory_id: impl Into<String>) -> Self {
        self.builder.push(" AND Transactions.SubcategoryId = ");
        self.builder.push_bind(subcategory_id.into());
        self
    }

    pub fn where_type(mut self, transaction_type: TransactionType) -> Self {
        self.builder.push(" AND TransactionType = ");
        self.builder.push_bind::<&str>(transaction_type.into());
        self
    }

    pub fn where_external_account(mut self, external_account_id: impl Into<String>) -> Self {
        self.builder.push(" AND ExternalAccountId = ");
        self.builder.push_bind(external_account_id.into());
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
            .push(" ORDER BY Date DESC, OrderIndicator DESC ");
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
            bank_account: BankAccountDto {
                id: record.bank_account_id,
                iban: record.bank_account_iban,
                name: record.bank_account_name,
                description: record.bank_account_description,
                hex_color: record.bank_account_hex_color,
            },
            category: None,
            subcategory: None,
            external_account_name: record.external_account_name,
            external_account: None,
        };

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

        if let Some(id) = record.external_account_associated_name {
            transaction.external_account = Some(ExternalAccountDto {
                id,
                name: record
                    .external_account_entity_name
                    .expect("External account id was not null, the the external account name was"),
                description: record.external_account_description.expect(
                    "External account id was not null, the the external account description was",
                ),
                default_category_id: record.external_account_default_category_id,
            })
        }

        transaction
    }
}
