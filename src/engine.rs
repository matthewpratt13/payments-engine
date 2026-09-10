use std::collections::HashMap;

use crate::account::Account;
use crate::transaction::TransactionRecord;
use crate::types::{ClientId, TransactionId};

#[derive(Debug)]
pub struct Engine {
    accounts: HashMap<ClientId, Account>,
    transactions: HashMap<TransactionId, TransactionRecord>,
}
