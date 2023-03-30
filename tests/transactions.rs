use rocket::serde::json::Json;
use sqlx::PgPool;
use ledgero_api::models::dto::pagination::pagination_query_dto::PaginationQueryDto;
use ledgero_api::models::dto::transactions::new_split_dto::NewSplitDto;
use ledgero_api::models::dto::transactions::transaction_set_category_dto::TransactionSetCategoryDto;
use ledgero_api::models::dto::transactions::update_transaction_dto::UpdateTransactionDto;
use ledgero_api::prelude::*;
use ledgero_api::routes::transactions::transaction_management::{change_category_for_transaction, get_all_transactions, get_single_transaction, update_transaction};
use crate::common::TestApp;

mod common;

#[sqlx::test(fixtures("users", "transactions"))]
async fn transactions_are_returned_correctly(pool: PgPool) {
    let app = TestApp::new(pool);

    let transactions = get_all_transactions(app.pool_state(), app.alice(), PaginationQueryDto {
        page: 1,
        limit: 10,
    })
        .await
        .unwrap()
        .0
        .into_items();

    assert_eq!(transactions.len(), 3);
    assert_eq!(transactions.get(0).unwrap().id, "transaction-1");
    assert_eq!(transactions.get(1).unwrap().id, "transaction-2");
    assert_eq!(transactions.get(2).unwrap().id, "transaction-3");
}

#[sqlx::test(fixtures("users", "transactions"))]
async fn transactions_can_be_paginated(pool: PgPool) {
    let app = TestApp::new(pool);

    let page = get_all_transactions(app.pool_state(), app.alice(), PaginationQueryDto {
        page: 1,
        limit: 2,
    })
        .await
        .unwrap()
        .0;

    assert!(!page.is_done());

    let transactions = page.into_items();

    assert_eq!(transactions.len(), 2);
    assert_eq!(transactions.get(0).unwrap().id, "transaction-1");
    assert_eq!(transactions.get(1).unwrap().id, "transaction-2");

    let page = get_all_transactions(app.pool_state(), app.alice(), PaginationQueryDto {
        page: 2,
        limit: 2,
    })
        .await
        .unwrap()
        .0;

    assert!(page.is_done());

    let transactions = page.into_items();

    assert_eq!(transactions.len(), 1);
    assert_eq!(transactions.get(0).unwrap().id, "transaction-3");
}

#[sqlx::test(fixtures("users", "transactions"))]
async fn a_single_transaction_is_returned_correctly(pool: PgPool) {
    let app = TestApp::new(pool);

    let transaction = get_single_transaction(
        app.pool_state(),
        app.alice(),
        "transaction-2"
    )
        .await
        .unwrap()
        .0;

    assert_eq!(transaction.id, "transaction-2");
    assert_eq!(transaction.amount, -9300);

    let category = transaction.category.unwrap();

    assert_eq!(category.id, "transaction-category-1");
    assert_eq!(category.name, "Groceries");
    assert_eq!(category.description, "For all the food");
    assert_eq!(category.hex_color, "303030");

    let bank_account = transaction.bank_account;

    assert_eq!(bank_account.id, "bank-account-1");
    assert_eq!(bank_account.iban, "NL12 RABO 12345678910");
    assert_eq!(bank_account.name, "Primary bank account");
    assert_eq!(bank_account.description, "For all of the normal stuff");
    assert_eq!(bank_account.hex_color, "ff3030");

    let external_account = transaction.external_account.unwrap();

    assert_eq!(external_account.id, "transaction-external-account-1");
    assert_eq!(external_account.name, "Jumbo");
    assert_eq!(external_account.description, "The price it quite high");
    assert_eq!(external_account.hex_color, "303030");
}

#[sqlx::test(fixtures("users", "transactions", "categories"))]
async fn the_category_of_a_transaction_can_be_changed(pool: PgPool) {
    let app = TestApp::new(pool);

    let transaction = change_category_for_transaction(
        app.pool_state(),
        app.alice(),
        "transaction-1",
        Json(TransactionSetCategoryDto {
            category_id: Some("category-1"),
            subcategory_id: None,
        })
    )
        .await
        .unwrap()
        .0;

    let category = transaction.category.unwrap();

    assert_eq!(category.id, "category-1");
    assert_eq!(category.name, "Groceries");
}

#[sqlx::test(fixtures("users", "transactions", "categories", "subcategories"))]
async fn the_subcategory_of_a_transaction_can_be_changed(pool: PgPool) {
    let app = TestApp::new(pool);

    let transaction = change_category_for_transaction(
        app.pool_state(),
        app.alice(),
        "transaction-1",
        Json(TransactionSetCategoryDto {
            category_id: Some("category-1"),
            subcategory_id: Some("subcategory-1"),
        })
    )
        .await
        .unwrap()
        .0;

    let category = transaction.category.unwrap();

    assert_eq!(category.id, "category-1");
    assert_eq!(category.name, "Groceries");

    let subcategory = transaction.subcategory.unwrap();

    assert_eq!(subcategory.id, "subcategory-1");
    assert_eq!(subcategory.name, "Subcategory 1");
}

#[sqlx::test(fixtures("users", "transactions", "categories", "subcategories"))]
async fn cannot_update_subcategory_without_category(pool: PgPool) {
    let app = TestApp::new(pool);

    let result = change_category_for_transaction(
        app.pool_state(),
        app.alice(),
        "transaction-1",
        Json(TransactionSetCategoryDto {
            category_id: None,
            subcategory_id: Some("subcategory-1"),
        })
    )
        .await;

    assert!(result.is_err());
}

#[sqlx::test(fixtures("users", "transactions", "categories", "subcategories"))]
async fn cannot_update_subcategory_with_a_category_that_is_not_parent(pool: PgPool) {
    let app = TestApp::new(pool);

    let result = change_category_for_transaction(
        app.pool_state(),
        app.alice(),
        "transaction-1",
        Json(TransactionSetCategoryDto {
            category_id: Some("category-1"),
            subcategory_id: Some("subcategory-2"),
        })
    )
        .await;

    assert!(result.is_err());
}

#[sqlx::test(fixtures("users", "transactions", "categories", "subcategories", "external-accounts"))]
async fn an_entire_transaction_can_be_updated(pool: PgPool) {
    let app = TestApp::new(pool);

    let transaction = update_transaction(
        app.pool_state(),
        app.alice(),
        "transaction-1",
        Json(UpdateTransactionDto {
            description: "A new description",
            category_id: Some("category-1"),
            subcategory_id: Some("subcategory-1"),
            external_account_id: Some("external-account-1"),
            splits: vec![],
        })
    )
        .await
        .unwrap()
        .0;

    // The description should be updated
    assert_eq!(transaction.description, "A new description");

    // The original description should always stay the same
    assert_eq!(transaction.original_description, "SALARY FROM WORK");

    let category = transaction.category.unwrap();

    assert_eq!(category.id, "category-1");
    assert_eq!(category.name, "Groceries");

    let subcategory = transaction.subcategory.unwrap();

    assert_eq!(subcategory.id, "subcategory-1");
    assert_eq!(subcategory.name, "Subcategory 1");

    let external_account = transaction.external_account.unwrap();

    assert_eq!(external_account.id, "external-account-1");
    assert_eq!(external_account.name, "Jumbo");
}

#[sqlx::test(fixtures("users", "transactions", "categories", "subcategories"))]
async fn subcategory_cannot_be_set_without_category_when_updating_entire_transaction(pool: PgPool) {
    let app = TestApp::new(pool);

    let result = update_transaction(
        app.pool_state(),
        app.alice(),
        "transaction-1",
        Json(UpdateTransactionDto {
            description: "A new description",
            category_id: None,
            subcategory_id: Some("subcategory-1"),
            external_account_id: None,
            splits: vec![],
        })
    )
        .await;

    assert!(result.is_err());
}

