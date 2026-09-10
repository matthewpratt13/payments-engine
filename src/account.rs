use rust_decimal::Decimal;
use serde::{Serialize, Serializer};

use crate::ClientId;

/// Represents an account that has been processed by the engine and can be output as a CSV record
#[derive(Debug, PartialEq, Eq, Serialize)]
pub struct AccountOutput {
    /// Unique client ID
    pub client: ClientId,
    /// Available funds (total - held)
    #[serde(serialize_with = "serialize_decimal")]
    pub available: Decimal,
    /// Held funds (not available for withdrawal)
    #[serde(serialize_with = "serialize_decimal")]
    pub held: Decimal,
    /// Total funds (available + held)
    #[serde(serialize_with = "serialize_decimal")]
    pub total: Decimal,
    /// Whether the account is locked (cannot process any more transactions)
    pub locked: bool,
}

/// Represents an account that has been processed by the engine
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Account {
    available: Decimal,
    held: Decimal,
    locked: bool,
}

impl Account {
    /// Deposits `amount` into the account.
    /// Does nothing if the amount is zero or the account is locked.
    pub fn deposit(&mut self, amount: Decimal) {
        self.available += amount;
    }

    /// Withdraws `amount` when sufficient available funds exist.
    /// Returns `false` and leaves the account unchanged otherwise.
    pub fn try_withdraw(&mut self, amount: Decimal) -> bool {
        if self.available < amount {
            return false;
        }

        self.available -= amount;

        true
    }

    /// Holds `amount` of funds, making them unavailable for withdrawal.
    /// Does nothing if the amount is zero or the account is locked.
    pub fn hold(&mut self, amount: Decimal) {
        self.available -= amount;
        self.held += amount;
    }

    /// Releases `amount` of held funds, making them available for withdrawal.
    /// Does nothing if the amount is zero or the account is locked.
    pub fn release(&mut self, amount: Decimal) {
        self.held -= amount;
        self.available += amount;
    }

    /// Charges back `amount` of held funds, making them unavailable for withdrawal.
    /// Does nothing if the amount is zero or the account is locked.
    pub fn chargeback(&mut self, amount: Decimal) {
        self.held -= amount;
        self.locked = true;
    }

    pub fn to_output(&self, client_id: ClientId) -> AccountOutput {
        AccountOutput {
            client: client_id,
            available: self.available,
            held: self.held,
            total: self.total(),
            locked: self.locked,
        }
    }

    pub fn is_locked(&self) -> bool {
        self.locked
    }

    pub fn total(&self) -> Decimal {
        self.available + self.held
    }
}

/// Custom serializer for [`Decimal`] to ensure 4 decimal places are always output
fn serialize_decimal<S>(value: &Decimal, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(&format!("{value:.4}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn amount(value: &str) -> Decimal {
        value.parse::<Decimal>().expect("valid Decimal value")
    }

    #[test]
    fn deposit_increases_available_and_total() {
        let mut account = Account::default();
        account.deposit(amount("1.5"));

        assert_eq!(account.available, amount("1.5"));
        assert_eq!(account.held, Decimal::ZERO);
        assert_eq!(account.total(), amount("1.5"));
        assert!(!account.is_locked());
    }

    #[test]
    fn successful_withdrawal_decreases_available() {
        let mut account = Account::default();
        account.deposit(amount("2.0"));

        assert!(account.try_withdraw(amount("1.5")));
        assert_eq!(account.available, amount("0.5"));
        assert_eq!(account.total(), amount("0.5"));
    }

    #[test]
    fn failed_withdrawal_does_not_change_account() {
        let mut account = Account::default();
        account.deposit(amount("1.0"));

        assert!(!account.try_withdraw(amount("1.0001")));
        assert_eq!(account.available, amount("1.0"));
        assert_eq!(account.held, Decimal::ZERO);
        assert_eq!(account.total(), amount("1.0"));
        assert!(!account.is_locked());
    }

    #[test]
    fn hold_moves_available_to_held() {
        let mut account = Account::default();
        account.deposit(amount("10"));
        account.hold(amount("4"));

        assert_eq!(account.available, amount("6"));
        assert_eq!(account.held, amount("4"));
        assert_eq!(account.total(), amount("10"));
    }

    #[test]
    fn release_moves_held_back_to_available() {
        let mut account = Account::default();
        account.deposit(amount("10"));
        account.hold(amount("4"));
        account.release(amount("4"));

        assert_eq!(account.available, amount("10"));
        assert_eq!(account.held, Decimal::ZERO);
        assert_eq!(account.total(), amount("10"));
    }

    #[test]
    fn chargeback_removes_held_funds_and_locks_account() {
        let mut account = Account::default();
        account.deposit(amount("10"));
        account.hold(amount("4"));
        account.chargeback(amount("4"));

        assert_eq!(account.available, amount("6"));
        assert_eq!(account.held, Decimal::ZERO);
        assert_eq!(account.total(), amount("6"));
        assert!(account.is_locked());
    }

    #[test]
    fn output_derives_total_from_available_and_held() {
        let mut account = Account::default();
        account.deposit(amount("3.25"));
        account.hold(amount("1.25"));

        let output = account.to_output(7);
        assert_eq!(output.client, 7);
        assert_eq!(output.available, amount("2.00"));
        assert_eq!(output.held, amount("1.25"));
        assert_eq!(output.total, amount("3.25"));
        assert!(!output.locked);
    }
}
