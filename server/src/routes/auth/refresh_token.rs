use chrono::{Months, Utc};
use rocket::http::Status;
use rocket::serde::json::Json;
use uuid::Uuid;

use crate::db_inner;
use crate::error::http_error::HttpError;
use crate::models::dto::auth::auth_response_dto::AuthResponseDto;
use crate::models::dto::auth::auth_response_dto::jwt_access_token_payload::JwtAccessTokenPayload;
use crate::models::dto::auth::jwt_refresh_dto::JwtRefreshDto;
use crate::models::jwt::jwt_refresh_payload::JwtRefreshPayload;
use crate::models::jwt::jwt_user_payload::JwtUserPayload;
use crate::prelude::*;
use crate::shared::{SharedJwtService, SharedPool};

#[post("/refresh", data = "<body>")]
pub async fn refresh(
    pool: &SharedPool,
    jwt_service: &SharedJwtService,
    body: Json<JwtRefreshDto<'_>>,
) -> Result<Json<AuthResponseDto>> {
    let pool = db_inner!(pool);
    let body = body.0;

    let (_, access_payload) =
        jwt_service.decode_access_token_unchecked::<JwtUserPayload>(body.access_token)?;
    info!("Token refresh attempt for '{}'", access_payload.username);

    let refresh_payload: JwtRefreshPayload = jwt_service.decode_refresh_token(body.refresh_token)?;
    debug!(
        "Attempting to refresh using grant id '{}'",
        refresh_payload.grant_id
    );

    trace!("Checking if the user for the grant still exists");
    let user = sqlx::query!(
        r#"
            SELECT Id
            FROM Users
            WHERE Id = $1;
        "#,
        access_payload.uuid
    )
        .fetch_optional(pool)
        .await?;

    let Some(_) = user else {
        debug!("No user was found for grant id '{}'", refresh_payload.grant_id);
        return Err(
            HttpError::from_status(Status::NotFound)
                .message("No user with the give id was found. The user might have been deleted")
                .into()
        );
    };

    trace!("Checking if the grant exists");
    let grant = sqlx::query!(
        r#"
            SELECT *
            FROM Grants
            WHERE Id = $1;
        "#,
        refresh_payload.grant_id
    )
        .fetch_optional(pool)
        .await?;

    let Some(grant) = grant else {
        debug!("No grant found with id '{}'", refresh_payload.grant_id);
        return Err(
            HttpError::from_status(Status::Unauthorized)
                .message("The given refresh token has been revoked")
                .into()
        );
    };

    let new_grant_id = Uuid::new_v4().to_string();
    let new_expire_at = (Utc::now() + Months::new(3)).to_rfc3339();

    debug!(
        "Updating grant '{}' with new id '{}' and expire time '{}'",
        grant.id, new_grant_id, new_expire_at
    );
    sqlx::query!(
        r#"
            UPDATE Grants
            SET Id = $2, ExpireAt = $3
            WHERE Id = $1;
        "#,
        grant.id,
        new_grant_id,
        new_expire_at
    )
        .execute(pool)
        .await?;

    trace!("Generating new access- and refresh tokens");
    let access_token = jwt_service.create_access_token(
        &access_payload.uuid,
        &access_payload
    )?;

    let refresh_token = jwt_service.create_refresh_token(
        &access_payload.uuid,
        JwtRefreshPayload {
            grant_id: new_grant_id,
        },
    )?;

    info!(
        "Successfully refreshed JWT access token for '{}'",
        access_payload.username
    );
    Ok(Json(AuthResponseDto::JwtAccessToken(JwtAccessTokenPayload {
        access_token,
        refresh_token,
        token_type: "bearer".to_string(),
        expires: jwt_service.get_access_token_seconds(),
    })))
}
