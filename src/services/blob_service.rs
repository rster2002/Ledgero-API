use std::fs;
use base64_url::base64;
use chrono::Utc;
use rocket::data::DataStream;
use crate::error::blob_error::BlobError;
use crate::prelude::*;
use crate::shared::{DbPool, PROJECT_DIRS, SharedPool};
use crate::utils::rand_string::rand_string;
use sqlx::types::time::OffsetDateTime;

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

        // This is the path the file is initially written to to infer the mime type to the token
        // can be generated.
        let temp_base = PROJECT_DIRS.get()
            .expect("Initialized")
            .cache_dir();

        fs::create_dir_all(&temp_base)?;
        let temp_file = temp_base
            .join(&id);

        stream.into_file(&temp_file)
            .await?;

        let Some(file_meta) = infer::get_from_path(&temp_file)? else {
            return Err(BlobError::NoMimeType.into());
        };

        let unconfirmed_root = PROJECT_DIRS.get()
            .expect("Initialized")
            .cache_dir()
            .join("unconfirmed");

        let mimetype = file_meta.mime_type();
        let token = format!("storage-{}-{}", base64_url::encode(mimetype), id);

        sqlx::query!(
            r#"
                INSERT INTO Blobs
                VALUES ($1, $2, $3, $4, null);
            "#,
            token,
            user_id.into(),
            mimetype,
            OffsetDateTime::from_unix_timestamp(Utc::now().timestamp())?,
        )
            .execute(pool)
            .await?;

        fs::create_dir_all(&unconfirmed_root)?;
        fs::rename(temp_file, unconfirmed_root.join(&token))?;

        Ok(token)
    }

    pub async fn confirm_token(&self, token: impl Into<String>) -> Result<()> {
        todo!()
    }
}
