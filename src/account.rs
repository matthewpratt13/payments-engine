use rust_decimal::Decimal;
use serde::Serialize;

use crate::types::ClientId;

#[derive(Debug, Default)]
pub struct Account {
    available: Decimal,
    held: Decimal,
    locked: bool,
}

#[derive(Debug, Serialize)]
pub struct AccountOutput {
    client: ClientId,
    available: Decimal,
    held: Decimal,
    total: Decimal,
    locked: bool,
}
