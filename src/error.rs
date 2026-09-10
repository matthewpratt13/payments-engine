use crate::TransactionId;

/// Errors that can occur when processing transactions
#[derive(Debug, thiserror::Error)]
pub enum EngineError {
    #[error("failed to read input: {0}")]
    Io(#[from] std::io::Error),
    #[error("invalid CSV: {0}")]
    Csv(#[from] csv::Error),
    #[error("missing input file path")]
    MissingPath,
    #[error("transaction {tx} is missing a required amount")]
    MissingAmount { tx: TransactionId },
}
