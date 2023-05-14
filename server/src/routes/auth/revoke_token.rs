use rocket::serde::json::Json;

use crate::db_inner;
use crate::models::dto::auth::revoke_dto::RevokeDto;
use crate::models::entities::grant::Grant;
use crate::models::jwt::jwt_refresh_payload::JwtRefreshPayload;
use crate::models::jwt::jwt_user_payload::JwtUserPayload;
use crate::prelude::*;
use crate::shared::{SharedJwtService, SharedPool};

#[post("/revoke", data = "<body>")]
pub async fn revoke(
    pool: &SharedPool,
    jwt_service: &SharedJwtService,
    body: Json<RevokeDto>,
) -> Result<()> {
    let pool = db_inner!(pool);

    let body = body.0;
    let refresh_payload: JwtRefreshPayload = jwt_service.decode_refresh_token(body.refresh_token)?;

    debug!("Revoking grant with id '{}'", refresh_payload.grant_id);
    Grant::delete_by_id(pool, refresh_payload.grant_id).await?;

    Ok(())
}

#[put("/revoke-all")]
pub async fn revoke_all(
    pool: &SharedPool,
    user: JwtUserPayload,
) -> Result<()> {
    let inner_pool = db_inner!(pool);

    debug!("Logging user {} out everywhere", user);
    sqlx::query!(
        r#"
            DELETE FROM Grants
            WHERE UserId = $1;
        "#,
        user.uuid
    )
        .execute(inner_pool)
        .await?;

    debug!("Logged out user {} everywhere", user);
    Ok(())
}
