use sqlx::Type;

/// Dictates the behaviour of the transaction and how is should be used.
#[derive(Debug, Type)]
#[sqlx(type_name = "color")] // only for Postgres to match a type definition
#[sqlx(rename_all = "lowercase")]
pub enum TransactionType {
    /// Indicates that the transaction should be considered real and is actually talking about
    /// money. This is what most of transaction should be.
    Transaction,

    /// A split is used to split a single transaction into multiple transactions which can then be
    /// used to organize it into a different category than the parent transaction.
    Split,

    /// This is not considered a real transaction on it's own, but is used when the real account
    /// balance for the user does not match the total balance in the application, for example
    /// when using the application for the first time. After that however, this should rarely be
    /// used as if this is used too often could be an indication of bad bookkeeping.
    Correction,

    /// This is a virtual transaction which main use is to move money between categories. It should
    /// not be used to indicate a move between real bank accounts, as that should be a real
    /// [TransactionType::Transaction].
    Move,
}
