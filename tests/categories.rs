use crate::common::TestApp;
use ledgero_api::models::dto::categories::new_category_dto::NewCategoryDto;
use ledgero_api::models::dto::categories::subcategories::new_subcategory_dto::NewSubcategoryDto;
use ledgero_api::routes::categories::ordering::category_ordering;
use ledgero_api::routes::categories::subcategories::{
    create_subcategory, get_subcategories, subcategory_by_id, update_subcategory,
};
use ledgero_api::routes::categories::{
    create_new_category, delete_category, get_all_categories, get_category_by_id, update_category,
};
use rocket::serde::json::Json;
use sqlx::PgPool;

mod common;

#[sqlx::test(fixtures("users"))]
async fn category_can_be_created(pool: PgPool) {
    let app = TestApp::new(pool);

    let returned_category = create_new_category(
        app.pool_state(),
        app.alice(),
        Json(NewCategoryDto {
            name: "Test category",
            description: "A category created by test",
            hex_color: "ff3030",
        }),
    )
    .await
    .unwrap()
    .0;

    assert_eq!(returned_category.name, "Test category");
    assert_eq!(returned_category.description, "A category created by test");
    assert_eq!(returned_category.hex_color, "ff3030");

    let stored_category = get_all_categories(app.pool_state(), app.alice())
        .await
        .unwrap()
        .0
        .swap_remove(0);

    assert_eq!(stored_category.name, "Test category");
    assert_eq!(stored_category.description, "A category created by test");
    assert_eq!(stored_category.hex_color, "ff3030");
}

#[sqlx::test(fixtures("users", "categories"))]
async fn categories_are_returned_correctly(pool: PgPool) {
    let app = TestApp::new(pool);

    let categories = get_all_categories(app.pool_state(), app.alice())
        .await
        .unwrap()
        .0;

    assert_eq!(categories.len(), 2);

    let category_1 = categories.get(0).unwrap();

    assert_eq!(category_1.id, "category-1");
    assert_eq!(category_1.name, "Groceries");
    assert_eq!(category_1.description, "For all the food");
    assert_eq!(category_1.hex_color, "303030");

    let category_2 = categories.get(1).unwrap();

    assert_eq!(category_2.id, "category-2");
    assert_eq!(category_2.name, "Groceries");
    assert_eq!(category_2.description, "For all the food");
    assert_eq!(category_2.hex_color, "303030");
}

#[sqlx::test(fixtures("users", "categories"))]
async fn category_by_id_is_returned_correctly(pool: PgPool) {
    let app = TestApp::new(pool);

    let category = get_category_by_id(app.pool_state(), app.alice(), "category-1")
        .await
        .unwrap()
        .0;

    assert_eq!(category.id, "category-1");
    assert_eq!(category.name, "Groceries");
    assert_eq!(category.description, "For all the food");
    assert_eq!(category.hex_color, "303030");
}

#[sqlx::test(fixtures("users", "categories"))]
async fn cannot_get_a_category_that_does_not_exist(pool: PgPool) {
    let app = TestApp::new(pool);

    let result = get_category_by_id(app.pool_state(), app.alice(), "does-not-exist").await;

    assert!(result.is_err());
}

#[sqlx::test(fixtures("users", "categories"))]
async fn category_details_can_be_updated(pool: PgPool) {
    let app = TestApp::new(pool);

    let returned_category = update_category(
        app.pool_state(),
        app.alice(),
        "category-1",
        Json(NewCategoryDto {
            name: "Updated name",
            description: "Updated description",
            hex_color: "ffffff",
        }),
    )
    .await
    .unwrap()
    .0;

    assert_eq!(returned_category.id, "category-1");
    assert_eq!(returned_category.name, "Updated name");
    assert_eq!(returned_category.description, "Updated description");
    assert_eq!(returned_category.hex_color, "ffffff");

    let stored_category = get_category_by_id(app.pool_state(), app.alice(), "category-1")
        .await
        .unwrap()
        .0;

    assert_eq!(stored_category.id, "category-1");
    assert_eq!(stored_category.name, "Updated name");
    assert_eq!(stored_category.description, "Updated description");
    assert_eq!(stored_category.hex_color, "ffffff");
}

#[sqlx::test(fixtures("users", "categories"))]
async fn categories_ordering_can_be_changed(pool: PgPool) {
    let app = TestApp::new(pool);

    category_ordering(
        app.pool_state(),
        app.alice(),
        Json(vec!["category-1".to_string(), "category-2".to_string()]),
    )
    .await
    .unwrap();

    let categories = get_all_categories(app.pool_state(), app.alice())
        .await
        .unwrap()
        .0;

    assert_eq!(categories.get(0).unwrap().id, "category-1");
    assert_eq!(categories.get(1).unwrap().id, "category-2");

    category_ordering(
        app.pool_state(),
        app.alice(),
        Json(vec!["category-2".to_string(), "category-1".to_string()]),
    )
    .await
    .unwrap();

    let categories = get_all_categories(app.pool_state(), app.alice())
        .await
        .unwrap()
        .0;

    assert_eq!(categories.get(0).unwrap().id, "category-2");
    assert_eq!(categories.get(1).unwrap().id, "category-1");
}

#[sqlx::test(fixtures("users", "categories"))]
async fn cannot_update_a_category_that_does_not_exist(pool: PgPool) {
    let app = TestApp::new(pool);

    let result = update_category(
        app.pool_state(),
        app.alice(),
        "does-not-exist",
        Json(NewCategoryDto {
            name: "Updated name",
            description: "Updated description",
            hex_color: "ffffff",
        }),
    )
    .await;

    assert!(result.is_err());
}

#[sqlx::test(fixtures("users", "categories"))]
async fn category_can_be_deleted(pool: PgPool) {
    let app = TestApp::new(pool);

    delete_category(app.pool_state(), app.alice(), "category-1")
        .await
        .unwrap();

    let categories = get_all_categories(app.pool_state(), app.alice())
        .await
        .unwrap()
        .0;

    assert_eq!(categories.len(), 1);
}

#[sqlx::test(fixtures("users", "categories"))]
async fn cannot_delete_a_category_that_does_not_exist(pool: PgPool) {
    let app = TestApp::new(pool);

    let result = delete_category(app.pool_state(), app.alice(), "does-not-exist").await;

    assert!(result.is_err());
}

#[sqlx::test(fixtures("users", "categories"))]
async fn subcategory_can_be_created(pool: PgPool) {
    let app = TestApp::new(pool);

    let returned_subcategory = create_subcategory(
        app.pool_state(),
        app.alice(),
        "category-1",
        Json(NewSubcategoryDto {
            name: "New subcategory",
            description: "New subcategory description",
            hex_color: "303030",
        }),
    )
    .await
    .unwrap()
    .0;

    assert_eq!(returned_subcategory.name, "New subcategory");
    assert_eq!(
        returned_subcategory.description,
        "New subcategory description"
    );
    assert_eq!(returned_subcategory.hex_color, "303030");

    let subcategories = get_subcategories(app.pool_state(), app.alice(), "category-1")
        .await
        .unwrap()
        .0;

    assert_eq!(subcategories.len(), 1);

    let stored_subcategory = subcategories.get(0).unwrap();

    assert_eq!(stored_subcategory.name, "New subcategory");
    assert_eq!(
        stored_subcategory.description,
        "New subcategory description"
    );
    assert_eq!(stored_subcategory.hex_color, "303030");
}

#[sqlx::test(fixtures("users", "categories", "subcategories"))]
async fn subcategories_are_returned_correctly(pool: PgPool) {
    let app = TestApp::new(pool);

    let subcategories = get_subcategories(app.pool_state(), app.alice(), "category-1")
        .await
        .unwrap()
        .0;

    assert_eq!(subcategories.len(), 1);

    let subcategory = subcategories.get(0).unwrap();

    assert_eq!(subcategory.id, "subcategory-1");
    assert_eq!(subcategory.name, "Subcategory 1");
    assert_eq!(subcategory.description, "Test subcategory");
    assert_eq!(subcategory.hex_color, "030303");
}

#[sqlx::test(fixtures("users", "categories", "subcategories"))]
async fn subcategory_by_id_is_returned_correctly(pool: PgPool) {
    let app = TestApp::new(pool);

    let subcategory =
        subcategory_by_id(app.pool_state(), app.alice(), "category-1", "subcategory-1")
            .await
            .unwrap()
            .0;

    assert_eq!(subcategory.id, "subcategory-1");
    assert_eq!(subcategory.name, "Subcategory 1");
    assert_eq!(subcategory.description, "Test subcategory");
    assert_eq!(subcategory.hex_color, "030303");
}

#[sqlx::test(fixtures("users", "categories", "subcategories"))]
async fn cannot_get_a_subcategory_that_does_not_exist(pool: PgPool) {
    let app = TestApp::new(pool);

    let result = subcategory_by_id(
        app.pool_state(),
        app.alice(),
        "category-1",
        "does-not-exist",
    )
    .await;

    assert!(result.is_err());
}

#[sqlx::test(fixtures("users", "categories", "subcategories"))]
async fn cannot_get_a_subcategory_with_category_that_does_not_exist(pool: PgPool) {
    let app = TestApp::new(pool);

    let result = subcategory_by_id(
        app.pool_state(),
        app.alice(),
        "does-not-exist",
        "subcategory-1",
    )
    .await;

    assert!(result.is_err());
}

#[sqlx::test(fixtures("users", "categories", "subcategories"))]
async fn subcategory_can_be_updated(pool: PgPool) {
    let app = TestApp::new(pool);

    let returned_subcategory = update_subcategory(
        app.pool_state(),
        app.alice(),
        "category-1",
        "subcategory-1",
        Json(NewSubcategoryDto {
            name: "Updated subcategory name",
            description: "Updated subcategory description",
            hex_color: "404040",
        }),
    )
    .await
    .unwrap()
    .0;

    assert_eq!(returned_subcategory.name, "Updated subcategory name");
    assert_eq!(
        returned_subcategory.description,
        "Updated subcategory description"
    );
    assert_eq!(returned_subcategory.hex_color, "404040");

    let stored_subcategory =
        subcategory_by_id(app.pool_state(), app.alice(), "category-1", "subcategory-1")
            .await
            .unwrap()
            .0;

    assert_eq!(stored_subcategory.name, "Updated subcategory name");
    assert_eq!(
        stored_subcategory.description,
        "Updated subcategory description"
    );
    assert_eq!(stored_subcategory.hex_color, "404040");
}

#[sqlx::test(fixtures("users", "categories"))]
async fn cannot_update_a_subcategory_that_does_not_exist(pool: PgPool) {
    let app = TestApp::new(pool);

    let result = update_subcategory(
        app.pool_state(),
        app.alice(),
        "category-1",
        "does-not-exist",
        Json(NewSubcategoryDto {
            name: "Updated subcategory name",
            description: "Updated subcategory description",
            hex_color: "404040",
        }),
    )
    .await;

    assert!(result.is_err());
}

#[sqlx::test(fixtures("users", "categories", "subcategories"))]
async fn cannot_update_a_subcategory_with_category_that_does_not_exist(pool: PgPool) {
    let app = TestApp::new(pool);

    let result = update_subcategory(
        app.pool_state(),
        app.alice(),
        "does-not-exist",
        "subcategory-1",
        Json(NewSubcategoryDto {
            name: "Updated subcategory name",
            description: "Updated subcategory description",
            hex_color: "404040",
        }),
    )
    .await;

    assert!(result.is_err());
}
