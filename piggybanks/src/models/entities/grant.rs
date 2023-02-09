use crate::prelude::*;
use crate::shared_types::DbPool;
use chrono::{DateTime, Months, Utc};
use entity_macro::{table_name, Entity};

use uuid::Uuid;

/// A grant is used to verify that the refresh token is still valid and may still be used to
/// generate a new JWT token. The grant is removed from the database in the following situations:
///
/// * The most common instance where the grant is removed from the database is when the user logs
///   out. The refresh token in question should not be able to generate any new JWT tokens so the
///   grant is revoked.
/// * The user decides to log out of all devices. In this case, all the grants for the given user
///   are removed from the database so any refresh tokens that might still be around cannot be used
///   to generate a new JWT token.
/// * The grant expires. This should happen at the same time the refresh token associated with the
///   grant would expire. This has no security benefit, but is used to ensure no tailing grants
///   remain in the database while the actual refresh token has long expired.
///
/// A grant expire time should be the same as the refresh token associated with it and so when
/// a token is refreshed, the grant expire time should also be updated to match.
#[derive(Debug, sqlx::FromRow, Entity)]
#[table_name("Grants")]
#[sqlx(rename_all = "PascalCase")]
pub struct Grant {
    /// Id of the grant. Is used in the refresh token to verify the existence of a grant when
    /// refreshing a JWT.
    pub id: String,

    /// The uuid of the user associated with the grant.
    pub user_id: String,

    /// [RFC 3339](https://www.rfc-editor.org/rfc/rfc3339) formatted datetime after which the grant
    /// is considered expired. Primarily used for database cleanup as the expire time that is
    /// usually checked is the one of the refresh token.
    pub expire_at: String,
}

impl Grant {
    /// Used to create a new instance of a grant. This does not however, add it to the database.
    /// To add the grant to the database, use [Grant::create].
    pub fn new(user_id: impl Into<String>) -> Self {
        let expire_utc = Utc::now() + Months::new(3);

        Self {
            id: Uuid::new_v4().to_string(),
            user_id: user_id.into(),
            expire_at: expire_utc.to_rfc3339(),
        }
    }

    pub fn set_expire_at(&mut self, expire_at: DateTime<Utc>) {
        self.expire_at = expire_at.to_rfc3339();
    }

    pub async fn update(&self, pool: &DbPool) -> Result<()> {
        sqlx::query!(
            r#"
                UPDATE Grants
                SET UserId = $1, ExpireAt = $2
            "#,
            self.user_id,
            self.expire_at
        )
        .execute(pool)
        .await?;

        Ok(())
    }

    pub async fn delete(&self, pool: &DbPool) -> Result<()> {
        Self::delete_by_id(pool, &self.id).await?;
        Ok(())
    }

    pub async fn delete_by_id(pool: &DbPool, id: impl Into<String>) -> Result<()> {
        sqlx::query!(
            r#"
                DELETE FROM Grants
                WHERE Id = $1;
            "#,
            id.into()
        )
        .execute(pool)
        .await?;

        Ok(())
    }
}
