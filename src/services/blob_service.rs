use std::fs;
use std::path::PathBuf;

use base64_url::base64;
use chrono::Utc;
use rocket::data::DataStream;
use sqlx::types::time::OffsetDateTime;

use crate::error::blob_error::BlobError;
use crate::prelude::*;
use crate::shared::{DbPool, PROJECT_DIRS, SharedPool};
use crate::utils::rand_string::rand_string;

/// The blob service handles everything regarding blobs and files. Creating and using a blob
/// consists of multiple steps:
///
/// 1. First, a blob is uploaded using the [upload_stream] method. The file is checked and then
///    stored in a temporary location. The function then returns a token that the caller can use
///    to confirm the blob.
/// 2. Next, the token may be provided in the body of a request, or in some other way. The token
///    is used to confirm the blob and makes the blob permanent and from that point the blob can
///    be returned using the [get_blob] or [get_blob_with_mimetypes] methods.
pub struct BlobService {
    max_blob_unconfirmed: u32,
    stream_to_root: PathBuf,
    unconfirmed_root: PathBuf,
    confirmed_root: PathBuf,
}

impl BlobService {
    pub fn new(max_blob_unconfirmed: u32) -> Result<Self> {
        // This is the path the file is initially written to to infer the mime type to the token
        // can be generated.
        let stream_to_root = PROJECT_DIRS.get()
            .expect("Initialized")
            .cache_dir()
            .to_path_buf();

        let unconfirmed_root = PROJECT_DIRS.get()
            .expect("Initialized")
            .cache_dir()
            .join("unconfirmed");

        let confirmed_root = PROJECT_DIRS.get()
            .expect("Initialized")
            .data_dir()
            .join("blobs");

        fs::create_dir_all(&stream_to_root)?;
        fs::create_dir_all(&unconfirmed_root)?;
        fs::create_dir_all(&confirmed_root)?;

        debug!("'{:?}' is the root to stream new files to", stream_to_root);
        debug!("'{:?}' is the root for unconfirmed blobs", unconfirmed_root);
        debug!("'{:?}' is the root for confirmed blobs", confirmed_root);

        Ok(Self {
            max_blob_unconfirmed,
            stream_to_root,
            unconfirmed_root,
            confirmed_root,
        })
    }

    /// Stores the contents of a stream and returns a token which can later be confirmed.
    pub async fn upload_stream(
        &self,
        user_id: impl Into<String>,
        pool: &DbPool,
        stream: DataStream<'_>
    ) -> Result<String> {
        let id = rand_string(32);
        debug!("Starting upload for '{}'", id);

        let temp_file = self.stream_to_root
            .join(&id);

        debug!("Streaming into '{:?}'", temp_file);
        stream.into_file(&temp_file)
            .await?;

        let Some(file_meta) = infer::get_from_path(&temp_file)? else {
            info!("Failed to infer mimetype for '{:?}'", temp_file);
            return Err(BlobError::NoMimeType.into());
        };

        let mimetype = file_meta.mime_type();
        debug!("Stream '{}' has '{}' as it's mimetype", id, mimetype);

        let token = format!("storage-{}-{}", base64_url::encode(mimetype), id);
        debug!("Creating blob with token '{}'", token);

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

        let target_location = self.unconfirmed_root
            .join(&token);

        debug!("Moving streamed file from '{:?}' to '{:?}'", temp_file, target_location);
        fs::rename(temp_file, target_location)?;

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
        debug!("Confirming blob with token '{}' for '{}'", token, user_id);

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

        if let None = record_option {
            return Err(BlobError::NoBlobToConfirm.into());
        }

        let file_path = self.confirmed_root
            .join(&token);

        if file_path.exists() {
            return Ok(token);
        }

        let unconfirmed_file_path = self.unconfirmed_root
            .join(&token);

        if !unconfirmed_file_path.exists() {
            return Err(BlobError::NoBlobToConfirm.into());
        }

        debug!("Move unconfirmed blob from '{:?}' to '{:?}'", unconfirmed_file_path, file_path);
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

    pub async fn cleanup(
        &self,
        pool: &DbPool,
    ) -> Result<()> {
        info!("Starting blob cleanup");

        let result = sqlx::query!(
            r#"
                DELETE FROM blobs
                WHERE confirmedat IS null AND EXTRACT(EPOCH FROM (now() - uploadedat)) > $1::bigint;
            "#,
            self.max_blob_unconfirmed as i64
        )
            .execute(pool)
            .await?;

        info!("Deleted {} blob records from the database", result.rows_affected());

        let records = sqlx::query!(
            r#"
                SELECT token,
                       COUNT(e.*) +
                       COUNT(u.*) AS "references"
                FROM blobs
                LEFT JOIN externalaccounts e on blobs.token = e.image
                LEFT JOIN users u on blobs.token = u.profileimage
                GROUP BY blobs.token;
            "#
        )
            .fetch_all(pool)
            .await?;

        Ok(())
    }
}
