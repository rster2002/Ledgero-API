use rocket::{Data, Route};
use rocket::serde::json::Json;
use crate::models::dto::blobs::blob_token_dto::BlobTokenDto;
use crate::prelude::*;

pub fn create_blob_routes() -> Vec<Route> {
    routes![

    ]
}

#[post("/", data="<stream>")]
pub async fn upload_blob(
    stream: Data<'_>
) -> Result<Json<BlobTokenDto>> {
    todo!()
}
