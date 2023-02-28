use rocket::form::validate::Contains;
use rocket::serde::json::Json;
use crate::models::jwt::jwt_user_payload::JwtUserPayload;
use crate::shared_types::SharedPool;
use crate::prelude::*;

#[patch("/ordering", data="<body>")]
pub async fn category_ordering(
    pool: &SharedPool,
    user: JwtUserPayload,
    body: Json<Vec<String>>,
) -> Result<()> {
    let inner_pool = pool.inner();
    let body = body.0;

    let records = sqlx::query!(
        r#"
            SELECT Id
            FROM Categories
            WHERE UserId = $1;
        "#,
        user.uuid
    )
        .fetch_all(inner_pool)
        .await?;

    if body.len() > records.len() {
        return Err(Error::generic("You cannot provide more ids than there are categories"));
    }

    for record in records {
        if !body.contains(&record.id) {
            return Err(Error::generic(format!("Missing category id '{}'", record.id)));
        }
    }

    let mut db_transaction = pool.begin()
        .await?;

    for (i, id) in body.into_iter().enumerate() {
        sqlx::query!(
            r#"
                UPDATE Categories
                SET OrderIndex = $3
                WHERE Id = $1 AND UserId = $2;
            "#,
            id,
            user.uuid,
            i as i32
        )
            .execute(&mut db_transaction)
            .await?;
    }

    Ok(())
}
