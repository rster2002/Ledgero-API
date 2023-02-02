pub mod transaction_type;

use chrono::Utc;
use sqlx::FromRow;
use crate::models::entities::transaction::transaction_type::TransactionType;

/// A single transaction of money.
#[derive(Debug, FromRow)]
#[sqlx(rename_all = "PascalCase")]
pub struct Transaction {
    pub id: String,
    pub user_id: String,

    pub transaction_type: TransactionType,

    /// A unique number for the transaction. This is different from the [id] in that this must be
    /// set when importing so that duplicate transaction are not accidentally imported twice.
    pub follow_number: String,

    /// The original description of the transaction. This should always match the description of
    /// the actual description of the transaction on the user's back account and should not be
    /// changed after creation.
    pub original_description: String,

    /// The description of the transaction. This is used to tell transactions apart from each-other.
    pub description: String,

    /// The actual amount of the complete transaction in euro cents, so 1,54 would be 156 in this
    /// field. The difference with [amount] is that amount may be changed by creating a split, while
    /// the complete amount should only be changed when there is an actual change to the transaction
    /// itself.
    pub complete_amount: i64,

    /// The current amount of the transaction. This may be changed by creating a split.
    pub amount: i64,

    /// [RFC 3339](https://www.rfc-editor.org/rfc/rfc3339) formatted datetime on which the
    /// transaction has taken place.
    pub date: String,

    /// The account id associated with the transaction.
    pub bank_account_id: String,

    /// The category this transaction belongs to. If the category id is [None] is is not part of
    /// a real category, but instead should be considered part of an "unsorted" category.
    pub category_id: Option<String>,

    /// The id of the parent transaction. This should be set for split transactions and should
    /// reference.
    pub parent_transaction_id: Option<String>,

    /// The name of the other account in this transaction. This is used to identify the external
    /// account, which may then be explicitly linked with the [external_account_id].
    pub external_account_name: String,

    /// The id referencing an external account entity. The [external_account_name] does not have
    /// to match with the actual name of the external account.
    pub external_account_id: Option<String>,
}
