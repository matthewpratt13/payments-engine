use rust_decimal::Decimal;
use serde::Deserialize;

use crate::error::EngineError;
use crate::{ClientId, TransactionId};

const DECIMAL_PLACES: u32 = 4;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DepositState {
    Settled,
    Disputed,
    ChargedBack,
}

/// Represents a transaction that can be processed by the engine
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Transaction {
    Deposit {
        client: ClientId,
        tx: TransactionId,
        amount: Decimal,
    },

    Withdrawal {
        client: ClientId,
        tx: TransactionId,
        amount: Decimal,
    },

    Dispute {
        client: ClientId,
        tx: TransactionId,
    },

    Resolve {
        client: ClientId,
        tx: TransactionId,
    },

    Chargeback {
        client: ClientId,
        tx: TransactionId,
    },
}

impl TryFrom<InputRecord> for Transaction {
    type Error = EngineError;

    fn try_from(value: InputRecord) -> Result<Self, Self::Error> {
        let InputRecord {
            kind,
            client,
            tx,
            amount,
        } = value;

        match kind {
            TransactionKind::Deposit => Ok(Transaction::Deposit {
                client,
                tx,
                amount: scale_amount(required_amount(tx, amount)?),
            }),
            TransactionKind::Withdrawal => Ok(Transaction::Withdrawal {
                client,
                tx,
                amount: scale_amount(required_amount(tx, amount)?),
            }),
            TransactionKind::Dispute => Ok(Transaction::Dispute { client, tx }),
            TransactionKind::Resolve => Ok(Transaction::Resolve { client, tx }),
            TransactionKind::Chargeback => Ok(Transaction::Chargeback { client, tx }),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
enum TransactionKind {
    Deposit,
    Withdrawal,
    Dispute,
    Resolve,
    Chargeback,
}

/// Represents a deposit that has been processed by the engine
#[derive(Debug)]
pub struct Deposit {
    pub(crate) client: ClientId,
    pub(crate) amount: Decimal,
    pub(crate) state: DepositState,
}

/// Represents a transaction that has been read from a CSV file
#[derive(Debug, Deserialize)]
pub struct InputRecord {
    #[serde(rename = "type")]
    kind: TransactionKind,
    client: ClientId,
    tx: TransactionId,
    amount: Option<Decimal>,
}

fn required_amount(
    tx_id: TransactionId,
    amount_opt: Option<Decimal>,
) -> Result<Decimal, EngineError> {
    amount_opt.ok_or(EngineError::MissingAmount { tx: tx_id })
}

fn scale_amount(amount: Decimal) -> Decimal {
    amount.round_dp(DECIMAL_PLACES)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input(
        tx_kind: TransactionKind,
        client_id: ClientId,
        tx_id: TransactionId,
        amount_opt: Option<&str>,
    ) -> InputRecord {
        InputRecord {
            kind: tx_kind,
            client: client_id,
            tx: tx_id,
            amount: amount_opt.map(|value| value.parse::<Decimal>().expect("valid Decimal value")),
        }
    }

    #[test]
    fn deposit_requires_amount() {
        let input = input(TransactionKind::Deposit, 1, 1, None);

        let error = Transaction::try_from(input).unwrap_err();
        assert!(matches!(error, EngineError::MissingAmount { tx: 1 }));
    }

    #[test]
    fn withdrawal_requires_amount() {
        let input = input(TransactionKind::Withdrawal, 1, 2, None);

        let error = Transaction::try_from(input).unwrap_err();
        assert!(matches!(error, EngineError::MissingAmount { tx: 2 }));
    }

    #[test]
    fn dispute_ignores_amount() {
        let input = input(TransactionKind::Dispute, 1, 1, Some("10.0"));

        let tx = Transaction::try_from(input).expect("dispute does not require an amount");

        assert_eq!(tx, Transaction::Dispute { client: 1, tx: 1 });
    }

    #[test]
    fn amounts_are_rounded_to_four_decimal_places() {
        let input = input(TransactionKind::Deposit, 1, 1, Some("1.23456"));

        let tx = Transaction::try_from(input).expect("deposit with amount");

        assert_eq!(
            tx,
            Transaction::Deposit {
                client: 1,
                tx: 1,
                amount: "1.2346".parse::<Decimal>().unwrap(),
            }
        );
    }
}
