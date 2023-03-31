use crate::common::TestApp;
use ledgero_api::models::dto::transactions::new_split_dto::NewSplitDto;
use ledgero_api::prelude::*;
use ledgero_api::routes::transactions::splits::{
    create_split, delete_split, get_splits, update_split,
};
use ledgero_api::routes::transactions::transaction_management::get_single_transaction;
use rocket::serde::json::Json;
use sqlx::PgPool;

mod common;

#[sqlx::test(fixtures("users", "transactions", "splits"))]
async fn all_splits_are_returned_correctly(pool: PgPool) {
    let app = TestApp::new(pool);

    let splits = get_splits(app.pool_state(), app.alice(), "transaction-1")
        .await
        .unwrap()
        .0;

    assert_eq!(splits.len(), 2);

    assert_eq!(splits.get(0).unwrap().id, "split-1");
    assert_eq!(splits.get(0).unwrap().description, "Allocated 1");

    assert_eq!(splits.get(1).unwrap().id, "split-2");
    assert_eq!(splits.get(1).unwrap().description, "Allocated 2");
}

#[sqlx::test(fixtures("users", "categories", "subcategories", "transactions"))]
async fn a_positive_split_can_be_created(pool: PgPool) {
    let app = TestApp::new(pool);

    create_split(
        app.pool_state(),
        app.alice(),
        "transaction-1",
        Json(NewSplitDto {
            description: "New split",
            amount: 10000,
            category_id: Some("category-1"),
            subcategory_id: Some("subcategory-1"),
        }),
    )
    .await
    .unwrap();

    let splits = get_splits(app.pool_state(), app.alice(), "transaction-1")
        .await
        .unwrap()
        .0;

    assert_eq!(splits.len(), 1);

    assert_eq!(splits.get(0).unwrap().description, "New split");

    util_check_transaction_amounts(&app, "transaction-1", 2567_00 - 100_00, 2567_00).await;
}

#[sqlx::test(fixtures("users", "transactions"))]
async fn multiple_positive_splits_can_be_created(pool: PgPool) {
    let app = TestApp::new(pool);

    create_split(
        app.pool_state(),
        app.alice(),
        "transaction-1",
        Json(NewSplitDto {
            description: "Split 1",
            amount: 100_00,
            category_id: None,
            subcategory_id: None,
        }),
    )
    .await
    .unwrap();

    create_split(
        app.pool_state(),
        app.alice(),
        "transaction-1",
        Json(NewSplitDto {
            description: "Split 2",
            amount: 50_00,
            category_id: None,
            subcategory_id: None,
        }),
    )
    .await
    .unwrap();

    util_check_transaction_amounts(&app, "transaction-1", 2567_00 - 150_00, 2567_00).await;
}

#[sqlx::test(fixtures("users", "transactions"))]
async fn multiple_negative_splits_can_be_created(pool: PgPool) {
    let app = TestApp::new(pool);

    create_split(
        app.pool_state(),
        app.alice(),
        "transaction-2",
        Json(NewSplitDto {
            description: "Split 1",
            amount: -30_00,
            category_id: None,
            subcategory_id: None,
        }),
    )
    .await
    .unwrap();

    create_split(
        app.pool_state(),
        app.alice(),
        "transaction-2",
        Json(NewSplitDto {
            description: "Split 2",
            amount: -15_00,
            category_id: None,
            subcategory_id: None,
        }),
    )
    .await
    .unwrap();

    util_check_transaction_amounts(&app, "transaction-2", -93_00 + 45_00, -93_00).await;
}

#[sqlx::test(fixtures("users", "categories", "transactions"))]
async fn cannot_create_a_negative_split_for_a_positive_transaction(pool: PgPool) {
    let app = TestApp::new(pool);

    let result = create_split(
        app.pool_state(),
        app.alice(),
        "transaction-1",
        Json(NewSplitDto {
            description: "New split",
            amount: -30_00,
            category_id: None,
            subcategory_id: None,
        }),
    )
    .await;

    assert!(result.is_err());
}

#[sqlx::test(fixtures("users", "categories", "subcategories", "transactions"))]
async fn a_negative_split_can_be_created(pool: PgPool) {
    let app = TestApp::new(pool);

    create_split(
        app.pool_state(),
        app.alice(),
        "transaction-2",
        Json(NewSplitDto {
            description: "New split",
            amount: -30_00,
            category_id: Some("category-1"),
            subcategory_id: Some("subcategory-1"),
        }),
    )
    .await
    .unwrap();

    let splits = get_splits(app.pool_state(), app.alice(), "transaction-2")
        .await
        .unwrap()
        .0;

    assert_eq!(splits.len(), 1);

    assert_eq!(splits.get(0).unwrap().description, "New split");

    util_check_transaction_amounts(&app, "transaction-2", -93_00 + 30_00, -93_00).await;
}

#[sqlx::test(fixtures("users", "categories", "transactions"))]
async fn cannot_create_a_positive_split_for_a_negative_transaction(pool: PgPool) {
    let app = TestApp::new(pool);

    let result = create_split(
        app.pool_state(),
        app.alice(),
        "transaction-2",
        Json(NewSplitDto {
            description: "New split",
            amount: 30_00,
            category_id: None,
            subcategory_id: None,
        }),
    )
    .await;

    assert!(result.is_err());

    let splits = get_splits(app.pool_state(), app.alice(), "transaction-2")
        .await
        .unwrap()
        .0;

    assert_eq!(splits.len(), 0);
}

#[sqlx::test(fixtures("users", "categories", "subcategories", "transactions"))]
async fn a_split_with_a_subcategory_cannot_be_created_if_the_category_is_not_set(pool: PgPool) {
    let app = TestApp::new(pool);

    let result = create_split(
        app.pool_state(),
        app.alice(),
        "transaction-1",
        Json(NewSplitDto {
            description: "New split",
            amount: 100_00,
            category_id: None,
            subcategory_id: Some("subcategory"),
        }),
    )
    .await;

    assert!(result.is_err());

    let splits = get_splits(app.pool_state(), app.alice(), "transaction-1")
        .await
        .unwrap()
        .0;

    assert_eq!(splits.len(), 0);
}

#[sqlx::test(fixtures("users", "transactions"))]
async fn cannot_create_a_single_split_that_exceeds_the_total_positive(pool: PgPool) {
    let app = TestApp::new(pool);

    let result = create_split(
        app.pool_state(),
        app.alice(),
        "transaction-1",
        Json(NewSplitDto {
            description: "Way too big",
            amount: 10000_00,
            category_id: None,
            subcategory_id: None,
        }),
    )
    .await;

    assert!(result.is_err());

    let splits = get_splits(app.pool_state(), app.alice(), "transaction-1")
        .await
        .unwrap()
        .0;

    assert_eq!(splits.len(), 0);

    util_check_transaction_amounts(&app, "transaction-1", 2567_00, 2567_00).await;
}

#[sqlx::test(fixtures("users", "transactions", "splits"))]
async fn cannot_create_a_single_split_that_exceeds_the_total_negative(pool: PgPool) {
    let app = TestApp::new(pool);

    let result = create_split(
        app.pool_state(),
        app.alice(),
        "transaction-2",
        Json(NewSplitDto {
            description: "Way too small",
            amount: -1000_00,
            category_id: None,
            subcategory_id: None,
        }),
    )
    .await;

    assert!(result.is_err());
}

#[sqlx::test(fixtures("users", "transactions"))]
async fn cannot_create_multiple_splits_that_exceeds_the_total_positive(pool: PgPool) {
    let app = TestApp::new(pool);

    create_split(
        app.pool_state(),
        app.alice(),
        "transaction-1",
        Json(NewSplitDto {
            description: "Way too big",
            amount: 1000_00,
            category_id: None,
            subcategory_id: None,
        }),
    )
    .await
    .unwrap();

    let result = create_split(
        app.pool_state(),
        app.alice(),
        "transaction-1",
        Json(NewSplitDto {
            description: "Way too big",
            amount: 10000_00,
            category_id: None,
            subcategory_id: None,
        }),
    )
    .await;

    assert!(result.is_err());

    let splits = get_splits(app.pool_state(), app.alice(), "transaction-1")
        .await
        .unwrap()
        .0;

    assert_eq!(splits.len(), 1);

    util_check_transaction_amounts(&app, "transaction-1", 2567_00 - 1000_00, 2567_00).await;
}

#[sqlx::test(fixtures("users", "transactions"))]
async fn cannot_create_multiple_splits_that_exceeds_the_total_negative(pool: PgPool) {
    let app = TestApp::new(pool);

    create_split(
        app.pool_state(),
        app.alice(),
        "transaction-2",
        Json(NewSplitDto {
            description: "Too low",
            amount: -3000,
            category_id: None,
            subcategory_id: None,
        }),
    )
    .await
    .unwrap();

    let result = create_split(
        app.pool_state(),
        app.alice(),
        "transaction-2",
        Json(NewSplitDto {
            description: "Too low",
            amount: -80_00,
            category_id: None,
            subcategory_id: None,
        }),
    )
    .await;

    assert!(result.is_err());

    let splits = get_splits(app.pool_state(), app.alice(), "transaction-2")
        .await
        .unwrap()
        .0;

    assert_eq!(splits.len(), 1);

    util_check_transaction_amounts(&app, "transaction-2", -93_00 + 30_00, -93_00).await;
}

#[sqlx::test(fixtures("users", "transactions", "splits"))]
async fn cannot_create_a_split_that_used_another_split_as_parent(pool: PgPool) {
    let app = TestApp::new(pool);

    let result = create_split(
        app.pool_state(),
        app.alice(),
        "split-1",
        Json(NewSplitDto {
            description: "That is not possible",
            amount: 1,
            category_id: None,
            subcategory_id: None,
        }),
    )
    .await;

    assert!(result.is_err());
}

#[sqlx::test(fixtures("users", "transactions", "splits"))]
async fn a_split_can_be_updated(pool: PgPool) {
    let app = TestApp::new(pool);

    update_split(
        app.pool_state(),
        app.alice(),
        "transaction-1",
        "split-1",
        Json(NewSplitDto {
            description: "Updated split",
            amount: 1000_00,
            category_id: None,
            subcategory_id: None,
        }),
    )
    .await
    .unwrap();

    let splits = get_splits(app.pool_state(), app.alice(), "transaction-1")
        .await
        .unwrap()
        .0;

    assert_eq!(splits.len(), 2);

    assert_eq!(splits.get(0).unwrap().description, "Updated split");

    util_check_transaction_amounts(&app, "transaction-1", 2567_00 - 1500_00, 2567_00).await;
}

#[sqlx::test(fixtures("users", "transactions", "splits"))]
async fn cannot_update_split_to_amount_zero(pool: PgPool) {
    let app = TestApp::new(pool);

    let result = update_split(
        app.pool_state(),
        app.alice(),
        "transaction-1",
        "split-1",
        Json(NewSplitDto {
            description: "Updated split",
            amount: 0,
            category_id: None,
            subcategory_id: None,
        }),
    )
    .await;

    assert!(result.is_err());
}

#[sqlx::test(fixtures("users", "transactions", "splits"))]
async fn cannot_update_a_split_that_doesnt_exist(pool: PgPool) {
    let app = TestApp::new(pool);

    let result = update_split(
        app.pool_state(),
        app.alice(),
        "transaction-1",
        "non-existent",
        Json(NewSplitDto {
            description: "Updated split",
            amount: 1000_00,
            category_id: None,
            subcategory_id: None,
        }),
    )
    .await;

    assert!(result.is_err());
}

#[sqlx::test(fixtures("users", "transactions", "splits"))]
async fn updating_a_positive_split_should_not_exceed_transaction_max(pool: PgPool) {
    let app = TestApp::new(pool);

    let result = update_split(
        app.pool_state(),
        app.alice(),
        "transaction-1",
        "split-1",
        Json(NewSplitDto {
            description: "Updated split",
            amount: 10000_00,
            category_id: None,
            subcategory_id: None,
        }),
    )
    .await;

    assert!(result.is_err());
}

#[sqlx::test(fixtures("users", "transactions", "splits"))]
async fn updating_a_negative_split_should_not_exceed_transaction_max(pool: PgPool) {
    let app = TestApp::new(pool);

    let result = update_split(
        app.pool_state(),
        app.alice(),
        "transaction-2",
        "split-3",
        Json(NewSplitDto {
            description: "Updated split",
            amount: -10000_00,
            category_id: None,
            subcategory_id: None,
        }),
    )
    .await;

    assert!(result.is_err());
}

#[sqlx::test(fixtures("users", "transactions", "splits"))]
async fn cannot_update_positive_split_with_negative_value(pool: PgPool) {
    let app = TestApp::new(pool);

    let result = update_split(
        app.pool_state(),
        app.alice(),
        "transaction-1",
        "split-1",
        Json(NewSplitDto {
            description: "Updated split",
            amount: -1_00,
            category_id: None,
            subcategory_id: None,
        }),
    )
    .await;

    assert!(result.is_err());
}

#[sqlx::test(fixtures("users", "transactions", "splits"))]
async fn cannot_update_negative_split_with_positive_value(pool: PgPool) {
    let app = TestApp::new(pool);

    let result = update_split(
        app.pool_state(),
        app.alice(),
        "transaction-2",
        "split-3",
        Json(NewSplitDto {
            description: "Updated split",
            amount: 1_00,
            category_id: None,
            subcategory_id: None,
        }),
    )
    .await;

    assert!(result.is_err());
}

#[sqlx::test(fixtures("users", "transactions", "splits"))]
async fn cannot_update_a_real_transaction_through_updating_a_split(pool: PgPool) {
    let app = TestApp::new(pool);

    let result = update_split(
        app.pool_state(),
        app.alice(),
        "transaction-1",
        "transaction-1",
        Json(NewSplitDto {
            description: "Updated split",
            amount: 1_00,
            category_id: None,
            subcategory_id: None,
        }),
    )
    .await;

    assert!(result.is_err());
}

#[sqlx::test(fixtures("users", "transactions", "splits"))]
async fn a_split_can_be_deleted(pool: PgPool) {
    let app = TestApp::new(pool);

    delete_split(app.pool_state(), app.alice(), "transaction-1", "split-1")
        .await
        .unwrap();

    util_check_transaction_amounts(&app, "transaction-1", 2567_00 - 500_00, 2567_00).await;
}

#[sqlx::test(fixtures("users", "transactions", "splits"))]
async fn cannot_delete_a_split_that_doesnt_exist(pool: PgPool) {
    let app = TestApp::new(pool);

    let result = delete_split(
        app.pool_state(),
        app.alice(),
        "transaction-1",
        "non-existent",
    )
    .await;

    assert!(result.is_err());
}

async fn util_check_transaction_amounts(
    app: &TestApp,
    transaction_id: &str,
    amount: i64,
    complete_amount: i64,
) {
    let transaction = get_single_transaction(app.pool_state(), app.alice(), transaction_id)
        .await
        .unwrap()
        .0;

    assert_eq!(transaction.amount, amount);
    assert_eq!(transaction.complete_amount, complete_amount);
}
