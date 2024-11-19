use bon::Builder;
use rust_decimal::Decimal;

#[derive(Debug, Clone, Builder, PartialEq)]
pub(crate) struct Tick {
    pub timestamp: i64,
    pub symbol: String,
    pub price: Decimal,
}
