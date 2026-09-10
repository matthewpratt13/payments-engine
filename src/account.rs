use rust_decimal::Decimal;

#[derive(Debug, Default)]
pub struct Account {
    available: Decimal,
    held: Decimal,
    locked: bool,
}
