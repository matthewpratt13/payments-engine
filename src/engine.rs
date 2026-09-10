use std::collections::HashMap;

use rust_decimal::Decimal;

use crate::account::{Account, AccountOutput};
use crate::transaction::{Transaction, TransactionRecord, TransactionState};
use crate::{ClientId, TransactionId};

/// Engine that processes transactions and maintains the account state
#[derive(Debug, Default)]
pub struct Engine {
    accounts: HashMap<ClientId, Account>,
    deposits: HashMap<TransactionId, TransactionRecord>,
}

impl Engine {
    pub fn new() -> Self {
        Engine::default()
    }

    /// Processes a transaction, updating the account state accordingly
    pub fn process_tx(&mut self, transaction: Transaction) {
        match transaction {
            Transaction::Deposit { client, tx, amount } => {
                self.process_deposit(client, tx, amount);
            }
            Transaction::Withdrawal { client, amount, .. } => {
                self.process_withdrawal(client, amount);
            }
            Transaction::Dispute { client, tx } => self.process_dispute(client, tx),
            Transaction::Resolve { client, tx } => self.process_resolve(client, tx),
            Transaction::Chargeback { client, tx } => self.process_chargeback(client, tx),
        }
    }

    /// Returns the current account state as a list of CSV records
    pub fn accounts(&self) -> Vec<AccountOutput> {
        let mut accounts: Vec<AccountOutput> = self
            .accounts
            .iter()
            .map(|(client_id, acc)| acc.to_output(*client_id))
            .collect();

        accounts.sort_by_key(|acc| acc.client);

        accounts
    }

    fn process_deposit(&mut self, client_id: ClientId, tx_id: TransactionId, amount: Decimal) {
        if amount <= Decimal::ZERO || self.deposits.contains_key(&tx_id) {
            return;
        }

        if self.account_is_locked(client_id) {
            return;
        }

        self.accounts.entry(client_id).or_default().deposit(amount);

        self.deposits.insert(
            tx_id,
            TransactionRecord {
                client: client_id,
                amount,
                state: TransactionState::Settled,
            },
        );
    }

    fn process_withdrawal(&mut self, client_id: ClientId, amount: Decimal) {
        if amount <= Decimal::ZERO {
            return;
        }

        let Some(account) = self.accounts.get_mut(&client_id) else {
            return;
        };

        if account.is_locked() {
            return;
        }

        account.try_withdraw(amount);
    }

    fn process_dispute(&mut self, client_id: ClientId, tx_id: TransactionId) {
        let Some(amount) = self.disputable_amount(client_id, tx_id, TransactionState::Settled)
        else {
            return;
        };

        if let Some(account) = self.accounts.get_mut(&client_id) {
            // Hold the original deposit even if those funds were later withdrawn,
            // which can make `available` negative
            account.hold(amount);
        }

        if let Some(deposit) = self.deposits.get_mut(&tx_id) {
            deposit.state = TransactionState::Disputed;
        }
    }

    fn process_resolve(&mut self, client_id: ClientId, tx_id: TransactionId) {
        let Some(amount) = self.disputable_amount(client_id, tx_id, TransactionState::Disputed)
        else {
            return;
        };

        if let Some(account) = self.accounts.get_mut(&client_id) {
            account.release(amount);
        }

        if let Some(deposit) = self.deposits.get_mut(&tx_id) {
            deposit.state = TransactionState::Settled;
        }
    }

    fn process_chargeback(&mut self, client_id: ClientId, tx_id: TransactionId) {
        let Some(amount) = self.disputable_amount(client_id, tx_id, TransactionState::Disputed)
        else {
            return;
        };

        if let Some(account) = self.accounts.get_mut(&client_id) {
            account.chargeback(amount);
        }

        if let Some(deposit) = self.deposits.get_mut(&tx_id) {
            deposit.state = TransactionState::ChargedBack;
        }
    }

    fn account_is_locked(&self, client_id: ClientId) -> bool {
        self.accounts
            .get(&client_id)
            .is_some_and(Account::is_locked)
    }

    fn disputable_amount(
        &self,
        client_id: ClientId,
        tx_id: TransactionId,
        expected_state: TransactionState,
    ) -> Option<Decimal> {
        let deposit = self.deposits.get(&tx_id)?;

        if deposit.client != client_id || deposit.state != expected_state {
            return None;
        }

        let account = self.accounts.get(&client_id)?;

        if account.is_locked() {
            return None;
        }

        Some(deposit.amount)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn amount(value: &str) -> Decimal {
        value.parse::<Decimal>().expect("valid Decimal value")
    }

    fn deposit(client_id: ClientId, tx_id: TransactionId, value: &str) -> Transaction {
        Transaction::Deposit {
            client: client_id,
            tx: tx_id,
            amount: amount(value),
        }
    }

    fn withdrawal(client_id: ClientId, tx_id: TransactionId, value: &str) -> Transaction {
        Transaction::Withdrawal {
            client: client_id,
            tx: tx_id,
            amount: amount(value),
        }
    }

    fn balances(engine: &Engine, client_id: ClientId) -> (Decimal, Decimal, Decimal, bool) {
        let output = engine
            .accounts()
            .into_iter()
            .find(|account| account.client == client_id)
            .expect("client account");

        (output.available, output.held, output.total, output.locked)
    }

    #[test]
    fn deposit_credits_available_funds() {
        let mut engine = Engine::new();
        engine.process_tx(deposit(1, 1, "1.0"));

        assert_eq!(
            balances(&engine, 1),
            (amount("1.0"), Decimal::ZERO, amount("1.0"), false)
        );
    }

    #[test]
    fn successful_and_failed_withdrawals() {
        let mut engine = Engine::new();
        engine.process_tx(deposit(1, 1, "1.0"));
        engine.process_tx(deposit(2, 2, "2.0"));
        engine.process_tx(deposit(1, 3, "2.0"));
        engine.process_tx(withdrawal(1, 4, "1.5"));
        engine.process_tx(withdrawal(2, 5, "3.0"));

        assert_eq!(
            balances(&engine, 1),
            (amount("1.5"), Decimal::ZERO, amount("1.5"), false)
        );

        assert_eq!(
            balances(&engine, 2),
            (amount("2.0"), Decimal::ZERO, amount("2.0"), false)
        );
    }

    #[test]
    fn dispute_resolve_and_chargeback() {
        let mut engine = Engine::new();
        engine.process_tx(deposit(1, 1, "10.0"));
        engine.process_tx(Transaction::Dispute { client: 1, tx: 1 });

        assert_eq!(
            balances(&engine, 1),
            (Decimal::ZERO, amount("10.0"), amount("10.0"), false)
        );

        engine.process_tx(Transaction::Resolve { client: 1, tx: 1 });

        assert_eq!(
            balances(&engine, 1),
            (amount("10.0"), Decimal::ZERO, amount("10.0"), false)
        );

        engine.process_tx(Transaction::Dispute { client: 1, tx: 1 });
        engine.process_tx(Transaction::Chargeback { client: 1, tx: 1 });

        assert_eq!(
            balances(&engine, 1),
            (Decimal::ZERO, Decimal::ZERO, Decimal::ZERO, true)
        );
    }

    #[test]
    fn invalid_references_are_ignored() {
        let mut engine = Engine::new();
        engine.process_tx(deposit(1, 1, "5.0"));
        engine.process_tx(Transaction::Dispute { client: 1, tx: 99 });
        engine.process_tx(Transaction::Resolve { client: 1, tx: 1 });
        engine.process_tx(Transaction::Chargeback { client: 1, tx: 1 });

        assert_eq!(
            balances(&engine, 1),
            (amount("5.0"), Decimal::ZERO, amount("5.0"), false)
        );
    }

    #[test]
    fn dispute_must_target_the_owning_client() {
        let mut engine = Engine::new();
        engine.process_tx(deposit(1, 1, "5.0"));
        engine.process_tx(Transaction::Dispute { client: 2, tx: 1 });

        assert_eq!(
            balances(&engine, 1),
            (amount("5.0"), Decimal::ZERO, amount("5.0"), false)
        );

        assert!(engine.accounts().iter().all(|row| row.client != 2));
    }

    #[test]
    fn invalid_state_transitions_are_ignored() {
        let mut engine = Engine::new();
        engine.process_tx(deposit(1, 1, "5.0"));
        engine.process_tx(Transaction::Resolve { client: 1, tx: 1 });
        engine.process_tx(Transaction::Chargeback { client: 1, tx: 1 });
        engine.process_tx(Transaction::Dispute { client: 1, tx: 1 });
        engine.process_tx(Transaction::Dispute { client: 1, tx: 1 });

        assert_eq!(
            balances(&engine, 1),
            (Decimal::ZERO, amount("5.0"), amount("5.0"), false)
        );
    }

    #[test]
    fn locked_account_ignores_later_transactions() {
        let mut engine = Engine::new();
        engine.process_tx(deposit(1, 1, "5.0"));
        engine.process_tx(deposit(1, 2, "3.0"));
        engine.process_tx(Transaction::Dispute { client: 1, tx: 1 });
        engine.process_tx(Transaction::Chargeback { client: 1, tx: 1 });
        engine.process_tx(deposit(1, 3, "10.0"));
        engine.process_tx(withdrawal(1, 4, "1.0"));
        engine.process_tx(Transaction::Dispute { client: 1, tx: 2 });
        engine.process_tx(Transaction::Resolve { client: 1, tx: 1 });

        assert_eq!(
            balances(&engine, 1),
            (amount("3.0"), Decimal::ZERO, amount("3.0"), true)
        );
    }

    #[test]
    fn multiple_clients_are_isolated() {
        let mut engine = Engine::new();
        engine.process_tx(deposit(2, 20, "2.0"));
        engine.process_tx(deposit(1, 10, "1.0"));

        let rows = engine.accounts();
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].client, 1);
        assert_eq!(rows[1].client, 2);
    }

    #[test]
    fn decimal_precision_is_preserved() {
        let mut engine = Engine::new();
        engine.process_tx(deposit(1, 1, "1.2345"));
        engine.process_tx(withdrawal(1, 2, "0.0005"));

        assert_eq!(
            balances(&engine, 1),
            (amount("1.2340"), Decimal::ZERO, amount("1.2340"), false)
        );
    }

    #[test]
    fn dispute_after_withdrawal_can_make_available_negative() {
        let mut engine = Engine::new();
        engine.process_tx(deposit(1, 1, "10.0"));
        engine.process_tx(withdrawal(1, 2, "10.0"));
        engine.process_tx(Transaction::Dispute { client: 1, tx: 1 });

        assert_eq!(
            balances(&engine, 1),
            (amount("-10.0"), amount("10.0"), Decimal::ZERO, false)
        );
    }

    #[test]
    fn duplicate_deposit_ids_are_ignored() {
        let mut engine = Engine::new();
        engine.process_tx(deposit(1, 1, "1.0"));
        engine.process_tx(deposit(1, 1, "9.0"));
        engine.process_tx(deposit(2, 1, "9.0"));

        assert_eq!(
            balances(&engine, 1),
            (amount("1.0"), Decimal::ZERO, amount("1.0"), false)
        );

        assert!(engine.accounts().iter().all(|row| row.client != 2));
    }

    #[test]
    fn zero_and_negative_amounts_are_ignored() {
        let mut engine = Engine::new();
        engine.process_tx(deposit(1, 1, "0"));
        engine.process_tx(deposit(1, 2, "-1.0"));
        engine.process_tx(deposit(1, 3, "5.0"));
        engine.process_tx(withdrawal(1, 4, "0"));
        engine.process_tx(withdrawal(1, 5, "-1.0"));

        assert_eq!(
            balances(&engine, 1),
            (amount("5.0"), Decimal::ZERO, amount("5.0"), false)
        );
    }

    #[test]
    fn withdrawals_cannot_be_disputed() {
        let mut engine = Engine::new();
        engine.process_tx(deposit(1, 1, "5.0"));
        engine.process_tx(withdrawal(1, 2, "1.0"));
        engine.process_tx(Transaction::Dispute { client: 1, tx: 2 });

        assert_eq!(
            balances(&engine, 1),
            (amount("4.0"), Decimal::ZERO, amount("4.0"), false)
        );
    }
}
