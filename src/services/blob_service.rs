use std::fs;
use base64_url::base64;
use chrono::Utc;
use rocket::data::DataStream;
use crate::error::blob_error::BlobError;
use crate::prelude::*;
use crate::shared::{DbPool, PROJECT_DIRS, SharedPool};
use crate::utils::rand_string::rand_string;
use sqlx::types::time::OffsetDateTime;

/// The blob service handles everything regarding blobs and files. Creating and using a blob
/// consists of multiple steps:
///
/// 1. First, a blob is uploaded using the [upload_stream] method. The file is checked and then
///    stored in a temporary location. The function then returns a token that the caller can use
///    to confirm the blob.
/// 2. Next, the token may be provided in the body of a request, or in some other way. The token
///    is used to confirm the blob and makes the blob permanent and from that point the blob can
///    be returned using the [get_blob] or [get_blob_with_mimetypes] methods.
pub struct BlobService;

impl BlobService {
    pub fn new() -> Self {
        Self {}
    }

    /// Stores the contents of a stream and returns a token which can later be confirmed.
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

    /// Does the same thing as the [confirm_token] method, but accepts an Option type. If the option
    /// has Some value the blob is confirmed and otherwise it's ignored.
    pub async fn confirm_optional(
        &self,
        user_id: impl Into<String>,
        pool: &DbPool,
        token: Option<impl Into<String>>,
    ) -> Result<Option<String>> {
        let Some(token) = token else {
            return Ok(None);
        };

        Ok(Some(self.confirm_token(
            user_id,
            pool,
            token
        ).await?))
    }

    /// Confirms that the blob is now used somewhere and should be available. There are a couple of
    /// rules when confirming an blob:
    ///
    /// * A user can only confirm their own blobs they've uploaded. This is to prevent other users
    ///   from also 'claiming' a blob where there are now two references to the same blob. This
    ///   could mean that if the original user wants to delete the image, the second user would
    ///   still have a reference to the blob causing it to persist.
    /// * If a token is provided that is already in use, the function should still return with Ok.
    ///   This is to help with updates. The above rule is important here, as another user can use
    ///   this rule to also 'claim' the blob.
    pub async fn confirm_token(
        &self,
        user_id: impl Into<String>,
        pool: &DbPool,
        token: impl Into<String>,
    ) -> Result<String> {
        let user_id = user_id.into();
        let token = token.into();

        let record_option = sqlx::query!(
            r#"
                SELECT Token
                FROM Blobs
                WHERE Token = $1 AND UserId = $2;
            "#,
            token,
            user_id
        )
            .fetch_optional(pool)
            .await?;

        let confirmed_root = PROJECT_DIRS.get()
            .expect("Initialized")
            .data_dir()
            .join("blobs");

        fs::create_dir_all(&confirmed_root)?;
        let file_path = confirmed_root.join(&token);

        if file_path.exists() {
            return Ok(token);
        }

        if let None = record_option {
            return Err(BlobError::NoBlobToConfirm.into());
        }

        let unconfirmed_file_path = PROJECT_DIRS.get()
            .expect("Initialized")
            .cache_dir()
            .join("unconfirmed")
            .join(&token);

        if !unconfirmed_file_path.exists() {
            return Err(BlobError::NoBlobToConfirm.into());
        }

        fs::rename(unconfirmed_file_path, file_path)?;

        sqlx::query!(
            r#"
                UPDATE Blobs
                SET ConfirmedAt = $3
                WHERE Token = $1 AND UserId = $2;
            "#,
            token,
            user_id,
            OffsetDateTime::from_unix_timestamp(Utc::now().timestamp())?,
        )
            .execute(pool)
            .await?;

        Ok(token)
    }

    /// Returns a stream for the given blob if it exists.
    pub async fn get_blob(&self, token: impl Into<String>) -> Result<()> {
        todo!()
    }

    /// Returns a stream for the given blob if it exists and matches at least one of the provided
    /// mime types.
    pub async fn get_blob_with_mimetype(&self, token: impl Into<String>) -> Result<()> {
        todo!()
    }

    pub async fn cleanup(&self) -> Result<()> {
        println!("Cleanup");
        Ok(())
    }
}
