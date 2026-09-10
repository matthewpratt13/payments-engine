use rust_decimal::Decimal;
use serde::Deserialize;

use crate::types::{ClientId, TransactionId};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TransactionKind {
    Deposit,
    Withdrawal,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TransactionState {
    Settled,
    Disputed,
    ChargedBack,
}

#[derive(Debug, Deserialize)]
pub struct InputRecord {
    #[serde(rename = "type")]
    kind: TransactionKind,

    client: ClientId,
    tx: TransactionId,
    amount: Option<Decimal>,
}

#[derive(Debug)]
pub struct TransactionRecord {
    client: ClientId,
    amount: Decimal,
    kind: TransactionKind,
    state: TransactionState,
}
