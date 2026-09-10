use crate::TransactionId;

/// Errors that can occur when processing transactions.
/// Only I/O errors are returned as the engine is expected to be used in a CLI context.
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
