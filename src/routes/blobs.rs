use directories::ProjectDirs;
use rocket::{Data, Route, State};
use rocket::data::ToByteUnit;
use rocket::serde::json::Json;
use crate::models::dto::blobs::blob_token_dto::BlobTokenDto;
use crate::models::jwt::jwt_user_payload::JwtUserPayload;
use crate::prelude::*;
use crate::services::blob_service::BlobService;
use crate::shared::{PROJECT_DIRS, SharedPool};

pub fn create_blob_routes() -> Vec<Route> {
    routes![
        upload_blob,
    ]
}

#[post("/upload", data="<stream>")]
pub async fn upload_blob(
    pool: &SharedPool,
    user: JwtUserPayload,
    blob_service: &State<BlobService>,
    stream: Data<'_>,
) -> Result<Json<BlobTokenDto>> {
    let inner_pool = pool.inner();
    let stream = stream.open(5.megabytes());

    let token = blob_service.inner()
        .upload_stream(user.uuid, inner_pool, stream)
        .await?;

    Ok(Json(BlobTokenDto {
        token,
    }))
}
