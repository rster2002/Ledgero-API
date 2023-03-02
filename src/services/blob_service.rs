use std::fs;
use chrono::Utc;
use rocket::data::DataStream;
use crate::error::blob_error::BlobError;
use crate::prelude::*;
use crate::shared::{DbPool, PROJECT_DIRS, SharedPool};
use crate::utils::rand_string::rand_string;

pub struct BlobService;

impl BlobService {
    pub fn new() -> Self {
        Self {}
    }

    pub async fn upload_stream(
        &self,
        user_id: impl Into<String>,
        pool: &DbPool,
        stream: DataStream<'_>
    ) -> Result<String> {
        let id = rand_string(32);

        let blobs_base = PROJECT_DIRS.get()
            .expect("Initialized")
            .data_dir()
            .join("blobs");

        fs::create_dir_all(&blobs_base)?;

        let file_path = blobs_base
            .join(&id);

        stream.into_file(&file_path)
            .await?;

        let Some(file_meta) = infer::get_from_path(&file_path)? else {
            return Err(BlobError::NoMimeType.into());
        };

        let result = sqlx::query!(
            r#"
                INSERT INTO Blobs
                VALUES ($1, $2, $3, $4);
            "#,
            id,
            user_id.into(),
            file_meta.mime_type(),
            false
        )
            .execute(pool)
            .await;

        if result.is_err() {
            fs::remove_file(&file_path)?;
            result?;
        }

        Ok(format!("storage-{}", id))
    }

    pub async fn confirm_token(&self, token: impl Into<String>) -> Result<()> {
        todo!()
    }
}
