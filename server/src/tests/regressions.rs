use rocket::serde::json::Json;
use sqlx::PgPool;

use crate::models::dto::categories::subcategories::new_subcategory_dto::NewSubcategoryDto;
use crate::routes::categories::get_all_categories;
use crate::routes::categories::subcategories::create_subcategory;
use crate::tests::common::TestApp;

#[sqlx::test(fixtures("users", "categories", "subcategories"))]
async fn regression_test_issue_17(pool: PgPool) {
    let app = TestApp::new(pool);

    create_subcategory(
        app.pool_state(),
        app.alice(),
        "category-1",
        Json(NewSubcategoryDto {
            name: "",
            description: "",
            hex_color: "",
        }),
    )
    .await
    .unwrap();

    create_subcategory(
        app.pool_state(),
        app.alice(),
        "category-1",
        Json(NewSubcategoryDto {
            name: "",
            description: "",
            hex_color: "",
        }),
    )
    .await
    .unwrap();

    let categories = get_all_categories(app.pool_state(), app.alice())
        .await
        .unwrap()
        .0;

    assert_eq!(categories.len(), 2);
}
