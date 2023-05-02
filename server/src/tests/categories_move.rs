use rocket::serde::json::Json;
use sqlx::PgPool;
use crate::models::dto::categories::move_between_categories_dto::MoveBetweenCategoriesDto;
use crate::models::dto::pagination::pagination_query_dto::PaginationQueryDto;
use crate::models::entities::transaction::transaction_type::TransactionType;
use crate::routes::categories::{get_category_by_id, get_category_transactions};
use crate::routes::categories::moving::{delete_move, move_money_between_categories};
use crate::routes::categories::subcategories::{get_subcategory_transactions, get_subcategory_by_id};
use crate::tests::common::TestApp;

#[sqlx::test(fixtures("users", "categories-move"))]
async fn money_can_be_moved_between_categories(pool: PgPool) {
    let app = TestApp::new(pool);

    // When
    move_money_between_categories(
        app.pool_state(),
        app.alice(),
        Json(MoveBetweenCategoriesDto {
            from_category_id: "category-A".to_string(),
            from_subcategory_id: None,
            to_category_id: "category-B".to_string(),
            to_subcategory_id: None,
            amount: 1000,
        })
    )
        .await
        .unwrap();

    // Then
    let category_a = get_category_by_id(
        app.pool_state(),
        app.alice(),
        "category-A"
    )
        .await
        .unwrap()
        .0;

    let category_b = get_category_by_id(
        app.pool_state(),
        app.alice(),
        "category-B"
    )
        .await
        .unwrap()
        .0;

    let transactions_a = get_category_transactions(
        app.pool_state(),
        app.alice(),
        "category-A".to_string(),
        PaginationQueryDto {
            page: 1,
            limit: 10,
        }
    )
        .await
        .unwrap()
        .0;

    let transactions_b = get_category_transactions(
        app.pool_state(),
        app.alice(),
        "category-B".to_string(),
        PaginationQueryDto {
            page: 1,
            limit: 10,
        }
    )
        .await
        .unwrap()
        .0;

    let move_transaction_a = transactions_a.into_items()
        .into_iter()
        .find(|transaction| transaction.transaction_type == TransactionType::Move)
        .unwrap();

    let move_transaction_b = transactions_b.into_items()
        .into_iter()
        .find(|transaction| transaction.transaction_type == TransactionType::Move)
        .unwrap();

    assert_eq!(category_a.amount, 0);
    assert_eq!(category_b.amount, 3000);

    assert_eq!(move_transaction_a.amount, -1000);
    assert_eq!(move_transaction_b.amount, 1000);

    assert_eq!(move_transaction_a.related_move_transaction.unwrap(), move_transaction_b.id);
    assert_eq!(move_transaction_b.related_move_transaction.unwrap(), move_transaction_a.id);
}

#[sqlx::test(fixtures("users", "categories-move"))]
async fn money_can_be_moved_between_subcategories(pool: PgPool) {
    let app = TestApp::new(pool);

    // When
    move_money_between_categories(
        app.pool_state(),
        app.alice(),
        Json(MoveBetweenCategoriesDto {
            from_category_id: "category-A".to_string(),
            from_subcategory_id: Some("subcategory-A".to_string()),
            to_category_id: "category-B".to_string(),
            to_subcategory_id: Some("subcategory-B".to_string()),
            amount: 1000,
        })
    )
        .await
        .unwrap();

    // Then
    let subcategory_a = get_subcategory_by_id(
        app.pool_state(),
        app.alice(),
        "category-A",
        "subcategory-A"
    )
        .await
        .unwrap()
        .0;

    let subcategory_b = get_subcategory_by_id(
        app.pool_state(),
        app.alice(),
        "category-B",
        "subcategory-B"
    )
        .await
        .unwrap()
        .0;

    let transactions_a = get_subcategory_transactions(
        app.pool_state(),
        app.alice(),
        "category-A",
        "subcategory-A",
        PaginationQueryDto {
            page: 1,
            limit: 10,
        }
    )
        .await
        .unwrap()
        .0;

    let transactions_b = get_subcategory_transactions(
        app.pool_state(),
        app.alice(),
        "category-B",
        "subcategory-B",
        PaginationQueryDto {
            page: 1,
            limit: 10,
        }
    )
        .await
        .unwrap()
        .0;

    let move_transaction_a = transactions_a.into_items()
        .into_iter()
        .find(|transaction| transaction.transaction_type == TransactionType::Move)
        .unwrap();

    let move_transaction_b = transactions_b.into_items()
        .into_iter()
        .find(|transaction| transaction.transaction_type == TransactionType::Move)
        .unwrap();

    assert_eq!(subcategory_a.amount, 0);
    assert_eq!(subcategory_b.amount, 3000);

    assert_eq!(move_transaction_a.amount, -1000);
    assert_eq!(move_transaction_b.amount, 1000);

    assert!(move_transaction_a.related_move_transaction.is_some());
    assert!(move_transaction_b.related_move_transaction.is_some());
    assert_eq!(move_transaction_a.related_move_transaction.unwrap(), move_transaction_b.id);
    assert_eq!(move_transaction_b.related_move_transaction.unwrap(), move_transaction_a.id);
}

#[sqlx::test]
async fn money_cannot_be_moved_to_a_category_that_does_not_exist(pool: PgPool) {
    let app = TestApp::new(pool);

    let result = move_money_between_categories(
        app.pool_state(),
        app.alice(),
        Json(MoveBetweenCategoriesDto {
            from_category_id: "category-A".to_string(),
            from_subcategory_id: None,
            to_category_id: "does-not-exist".to_string(),
            to_subcategory_id: None,
            amount: 1000,
        })
    )
        .await;

    assert!(result.is_err());
}

#[sqlx::test]
async fn money_cannot_be_moved_from_a_category_that_does_not_exist(pool: PgPool) {
    let app = TestApp::new(pool);

    let result = move_money_between_categories(
        app.pool_state(),
        app.alice(),
        Json(MoveBetweenCategoriesDto {
            from_category_id: "does-not-exist".to_string(),
            from_subcategory_id: None,
            to_category_id: "category-B".to_string(),
            to_subcategory_id: None,
            amount: 1000,
        })
    )
        .await;

    assert!(result.is_err());
}

#[sqlx::test]
async fn money_cannot_be_moved_to_a_subcategory_that_does_not_exist(pool: PgPool) {
    let app = TestApp::new(pool);

    let result = move_money_between_categories(
        app.pool_state(),
        app.alice(),
        Json(MoveBetweenCategoriesDto {
            from_category_id: "category-A".to_string(),
            from_subcategory_id: None,
            to_category_id: "category-B".to_string(),
            to_subcategory_id: Some("does-not-exist".to_string()),
            amount: 1000,
        })
    )
        .await;

    assert!(result.is_err());
}

#[sqlx::test]
async fn money_cannot_be_moved_from_a_subcategory_that_does_not_exist(pool: PgPool) {
    let app = TestApp::new(pool);

    let result = move_money_between_categories(
        app.pool_state(),
        app.alice(),
        Json(MoveBetweenCategoriesDto {
            from_category_id: "category-A".to_string(),
            from_subcategory_id: Some("does-not-exist".to_string()),
            to_category_id: "category-B".to_string(),
            to_subcategory_id: None,
            amount: 1000,
        })
    )
        .await;

    assert!(result.is_err());
}

#[sqlx::test(fixtures("users", "categories-move"))]
async fn move_transaction_can_be_deleted(pool: PgPool) {
    let app = TestApp::new(pool);

    move_money_between_categories(
        app.pool_state(),
        app.alice(),
        Json(MoveBetweenCategoriesDto {
            from_category_id: "category-A".to_string(),
            from_subcategory_id: None,
            to_category_id: "category-B".to_string(),
            to_subcategory_id: None,
            amount: 1000,
        })
    )
        .await
        .unwrap();

    let transactions_a = get_category_transactions(
        app.pool_state(),
        app.alice(),
        "category-A".to_string(),
        PaginationQueryDto {
            page: 1,
            limit: 10,
        }
    )
        .await
        .unwrap()
        .0
        .into_items()
        .into_iter()
        .find(|transaction| transaction.transaction_type == TransactionType::Move)
        .unwrap();

    delete_move(
        app.pool_state(),
        app.alice(),
        transactions_a.id
    )
        .await
        .unwrap();

    let move_a_is_none = get_category_transactions(
        app.pool_state(),
        app.alice(),
        "category-A".to_string(),
        PaginationQueryDto {
            page: 1,
            limit: 10,
        }
    )
        .await
        .unwrap()
        .0
        .into_items()
        .into_iter()
        .find(|transaction| transaction.transaction_type == TransactionType::Move)
        .is_none();

    let move_b_is_none = get_category_transactions(
        app.pool_state(),
        app.alice(),
        "category-B".to_string(),
        PaginationQueryDto {
            page: 1,
            limit: 10,
        }
    )
        .await
        .unwrap()
        .0
        .into_items()
        .into_iter()
        .find(|transaction| transaction.transaction_type == TransactionType::Move)
        .is_none();

    assert!(move_a_is_none);
    assert!(move_b_is_none);
}

#[sqlx::test(fixtures("users", "categories-move"))]
async fn cannot_delete_transaction_that_is_not_a_move(pool: PgPool) {
    let app = TestApp::new(pool);

    let result = delete_move(
        app.pool_state(),
        app.alice(),
        "move-transaction-1".to_string(),
    )
        .await;

    assert!(result.is_err());
}

#[sqlx::test(fixtures("users", "categories-move"))]
async fn cannot_delete_move_transaction_that_does_not_exist(pool: PgPool) {
    let app = TestApp::new(pool);

    let result = delete_move(
        app.pool_state(),
        app.alice(),
        "does-not-exist".to_string(),
    )
        .await;

    assert!(result.is_err());
}

