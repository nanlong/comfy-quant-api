use bon::Builder;
use rust_decimal::Decimal;

#[derive(Debug, Clone, Builder, PartialEq)]
pub(crate) struct Tick {
    pub(crate) timestamp: i64,
    pub(crate) price: Decimal,
}
