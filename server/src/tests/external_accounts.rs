use rocket::serde::json::Json;
use sqlx::PgPool;
use crate::models::dto::external_accounts::new_external_account_dto::NewExternalAccountDto;
use crate::models::dto::external_accounts::new_external_account_name_dto::NewExternalAccountNameDto;
use crate::models::dto::pagination::pagination_query_dto::PaginationQueryDto;
use crate::routes::external_accounts::{add_external_account_name, apply_external_account_name, create_new_external_account, delete_external_account, delete_external_account_name, get_all_external_accounts, get_external_account_by_id, get_external_account_names, get_transactions_for_external_account, remove_external_account_name_associations, update_external_account};
use crate::routes::transactions::transaction_management::{get_all_transactions, get_single_transaction};
use crate::tests::common::TestApp;

#[sqlx::test(fixtures("users", "external-accounts"))]
async fn all_external_accounts_are_returned_correctly(pool: PgPool) {
    let app = TestApp::new(pool);

    let external_accounts = get_all_external_accounts(
        app.pool_state(),
        app.alice(),
    )
        .await
        .unwrap()
        .0;

    assert_eq!(external_accounts.len(), 2);

    let first_external_account = external_accounts.get(0).unwrap();
    assert_eq!(first_external_account.id, "external-account-1");
    assert_eq!(first_external_account.name, "Jumbo");
    assert_eq!(first_external_account.description, "The price it quite high");
    assert_eq!(first_external_account.default_category_id, None);
    assert_eq!(first_external_account.default_subcategory_id, None);

    let second_external_account = external_accounts.get(1).unwrap();
    assert_eq!(second_external_account.id, "external-account-3");
    assert_eq!(second_external_account.name, "Evil landlord");
    assert_eq!(second_external_account.description, "The *******");
    assert_eq!(second_external_account.default_category_id, Some("category-external-account".to_string()));
    assert_eq!(second_external_account.default_subcategory_id, Some("subcategory-external-account".to_string()));
}

#[sqlx::test(fixtures("users", "external-accounts"))]
async fn single_external_account_is_returned_correctly(pool: PgPool) {
    let app = TestApp::new(pool);

    let external_account = get_external_account_by_id(
        app.pool_state(),
        app.alice(),
        "external-account-1".to_string()
    )
        .await
        .unwrap()
        .0;

    assert_eq!(external_account.id, "external-account-1");
    assert_eq!(external_account.name, "Jumbo");
    assert_eq!(external_account.description, "The price it quite high");
    assert_eq!(external_account.default_category_id, None);
    assert_eq!(external_account.default_subcategory_id, None);
}

#[sqlx::test(fixtures("users", "external-accounts"))]
async fn single_external_account_that_is_not_owned_by_used_is_not_returned(pool: PgPool) {
    let app = TestApp::new(pool);

    let result = get_external_account_by_id(
        app.pool_state(),
        app.alice(),
        "external-account-2".to_string()
    )
        .await;

    assert!(result.is_err());
}

#[sqlx::test(fixtures("users", "external-accounts"))]
async fn external_account_can_be_created(pool: PgPool) {
    let app = TestApp::new(pool);

    create_new_external_account(
        app.pool_state(),
        app.alice(),
        Json(NewExternalAccountDto {
            name: "Test",
            description: "Test description",
            hex_color: "3030ff",
            default_category_id: Some("category-external-account"),
            default_subcategory_id: Some("subcategory-external-account"),
        })
    )
        .await
        .unwrap();

    let external_accounts = get_all_external_accounts(
        app.pool_state(),
        app.alice(),
    )
        .await
        .unwrap()
        .0;

    let external_account = external_accounts
        .iter()
        .find(|account| account.name == "Test".to_string())
        .unwrap();

    dbg!(&external_account);

    assert_eq!(external_account.name, "Test");
    assert_eq!(external_account.description, "Test description");
    assert_eq!(external_account.hex_color, "3030ff");
    assert_eq!(external_account.default_category_id, Some("category-external-account".to_string()));
    assert_eq!(external_account.default_subcategory_id, Some("subcategory-external-account".to_string()));
}

#[sqlx::test(fixtures("users", "external-accounts"))]
async fn external_account_can_be_updated(pool: PgPool) {
    let app = TestApp::new(pool);

    update_external_account(
        app.pool_state(),
        app.alice(),
        "external-account-1".to_string(),
        Json(NewExternalAccountDto {
            name: "Albert Heijn",
            description: "Even worst",
            hex_color: "00a0e2",
            default_category_id: Some("category-external-account"),
            default_subcategory_id: Some("subcategory-external-account"),
        })
    )
        .await
        .unwrap();

    let external_account = get_external_account_by_id(
        app.pool_state(),
        app.alice(),
        "external-account-1".to_string(),
    )
        .await
        .unwrap()
        .0;

    assert_eq!(external_account.id, "external-account-1");
    assert_eq!(external_account.name, "Albert Heijn");
    assert_eq!(external_account.description, "Even worst");
    assert_eq!(external_account.hex_color, "00a0e2");
    assert_eq!(external_account.default_category_id, Some("category-external-account".to_string()));
    assert_eq!(external_account.default_subcategory_id, Some("subcategory-external-account".to_string()));
}

#[sqlx::test(fixtures("users", "external-accounts"))]
async fn external_account_can_be_deleted(pool: PgPool) {
    let app = TestApp::new(pool);

    delete_external_account(
        app.pool_state(),
        app.alice(),
        "external-account-1".to_string(),
    )
        .await
        .unwrap();

    let external_accounts = get_all_external_accounts(
        app.pool_state(),
        app.alice(),
    )
        .await
        .unwrap()
        .0;

    assert_eq!(external_accounts.len(), 1);

    let still_exists = external_accounts.iter()
        .find(|account| account.name == "Jumbo".to_string());

    assert!(still_exists.is_none());
}

#[sqlx::test(fixtures("users", "external-accounts"))]
async fn name_can_be_queried(pool: PgPool) {
    let app = TestApp::new(pool);

    let names = get_external_account_names(
        app.pool_state(),
        app.alice(),
        "external-account-3".to_string(),
    )
        .await
        .unwrap()
        .0;

    assert_eq!(names.len(), 1);
    assert_eq!(names.get(0).unwrap().name, "EVIL_LAND_LORD_INC");
}

#[sqlx::test(fixtures("users", "external-accounts"))]
async fn name_can_be_added_to_external_account(pool: PgPool) {
    let app = TestApp::new(pool);

    add_external_account_name(
        app.pool_state(),
        app.alice(),
        "external-account-1".to_string(),
        Json(NewExternalAccountNameDto {
            name: "jumbo"
        }),
    )
        .await
        .unwrap();

    let names = get_external_account_names(
        app.pool_state(),
        app.alice(),
        "external-account-1".to_string(),
    )
        .await
        .unwrap()
        .0;

    dbg!(&names);

    assert_eq!(names.len(), 1);
    assert_eq!(names.get(0).unwrap().name, "jumbo");
}

#[sqlx::test(fixtures("users", "external-accounts"))]
async fn name_can_be_removed(pool: PgPool) {
    let app = TestApp::new(pool);

    delete_external_account_name(
        app.pool_state(),
        app.alice(),
        "external-account-3".to_string(),
        "external-account-name-1".to_string(),
    )
        .await
        .unwrap();

    let names = get_external_account_names(
        app.pool_state(),
        app.alice(),
        "external-account-3".to_string(),
    )
        .await
        .unwrap()
        .0;

    assert_eq!(names.len(), 0);
}

#[sqlx::test(fixtures("users", "external-accounts", "transactions"))]
async fn external_account_name_can_be_applied_to_existing_transactions(pool: PgPool) {
    let app = TestApp::new(pool);

    apply_external_account_name(
        app.pool_state(),
        app.alice(),
        "external-account-3".to_string(),
        "external-account-name-1".to_string()
    )
        .await
        .unwrap();

    let transaction = get_single_transaction(
        app.pool_state(),
        app.alice(),
        "transaction-3"
    )
        .await
        .unwrap()
        .0;

    let external_account = transaction.external_account.unwrap();
    assert_eq!(external_account.id, "external-account-3");
    assert_eq!(external_account.name, "Evil landlord");
    assert_eq!(external_account.description, "The *******");
    assert_eq!(external_account.default_category_id, Some("category-external-account".to_string()));
    assert_eq!(external_account.default_subcategory_id, Some("subcategory-external-account".to_string()));
}

#[sqlx::test(fixtures("users", "external-accounts", "transactions"))]
async fn external_account_name_can_be_removed_from_existing_transactions(pool: PgPool) {
    let app = TestApp::new(pool);

    apply_external_account_name(
        app.pool_state(),
        app.alice(),
        "external-account-3".to_string(),
        "external-account-name-1".to_string()
    )
        .await
        .unwrap();

    get_single_transaction(
        app.pool_state(),
        app.alice(),
        "transaction-3"
    )
        .await
        .unwrap()
        .0
        .external_account
        .unwrap();

    remove_external_account_name_associations(
        app.pool_state(),
        app.alice(),
        "external-account-3".to_string(),
        "external-account-name-1".to_string()
    )
        .await
        .unwrap();

    let transaction = get_single_transaction(
        app.pool_state(),
        app.alice(),
        "transaction-3"
    )
        .await
        .unwrap()
        .0;

    assert!(transaction.external_account.is_none());
}

#[sqlx::test(fixtures("users", "external-accounts", "transactions"))]
async fn transactions_are_returned_correctly_for_external_account(pool: PgPool) {
    let app = TestApp::new(pool);

    apply_external_account_name(
        app.pool_state(),
        app.alice(),
        "external-account-3".to_string(),
        "external-account-name-1".to_string()
    )
        .await
        .unwrap();

    let transactions = get_transactions_for_external_account(
        app.pool_state(),
        app.alice(),
        "external-account-3".to_string(),
        PaginationQueryDto {
            page: 1,
            limit: 10,
        }
    )
        .await
        .unwrap()
        .0
        .into_items();

    assert_eq!(transactions.len(), 1);
}
