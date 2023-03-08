use sqlx::PgPool;
use ledgero_api::models::dto::pagination::pagination_query_dto::PaginationQueryDto;
use ledgero_api::prelude::*;
use ledgero_api::routes::transactions::transaction_management::{get_all_transactions, get_single_transaction};
use crate::common::TestApp;

mod common;

#[sqlx::test(fixtures("users", "transactions"))]
async fn transactions_are_returned_correctly(pool: PgPool) -> Result<()> {
    let app = TestApp::new(pool);

    let transactions = get_all_transactions(app.pool_state(), app.alice(), PaginationQueryDto {
        page: 1,
        limit: 10,
    })
        .await?
        .0
        .into_items();

    assert_eq!(transactions.len(), 3);
    assert_eq!(transactions.get(0).unwrap().id, "transaction-1");
    assert_eq!(transactions.get(1).unwrap().id, "transaction-2");
    assert_eq!(transactions.get(2).unwrap().id, "transaction-3");

    Ok(())
}

#[sqlx::test(fixtures("users", "transactions"))]
async fn transactions_can_be_paginated(pool: PgPool) -> Result<()> {
    let app = TestApp::new(pool);

    let page = get_all_transactions(app.pool_state(), app.alice(), PaginationQueryDto {
        page: 1,
        limit: 2,
    })
        .await?
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
        .await?
        .0;

    assert!(page.is_done());

    let transactions = page.into_items();

    assert_eq!(transactions.len(), 1);
    assert_eq!(transactions.get(0).unwrap().id, "transaction-3");

    Ok(())
}

#[sqlx::test(fixtures("users", "transactions"))]
async fn a_single_transaction_is_returned_correctly(pool: PgPool) -> Result<()> {
    let app = TestApp::new(pool);

    let transaction = get_single_transaction("transaction-1".to_string(), app.pool_state(), app.alice())
        .await?
        .0;

    assert_eq!(transaction.id, "transaction-1");
    assert_eq!(transaction.amount, 256700);

    Ok(())
}

#[sqlx::test(fixtures("users", "transactions"))]
async fn the_category_of_a_transaction_can_be_changed(pool: PgPool) -> Result<()> {
    todo!()
}

#[sqlx::test(fixtures("users", "transactions"))]
async fn an_entire_transaction_can_be_updated(pool: PgPool) -> Result<()> {
    todo!()
}

