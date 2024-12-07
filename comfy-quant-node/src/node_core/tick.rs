use bon::Builder;
use comfy_quant_exchange::client::spot_client::base::SymbolPrice;
use rust_decimal::Decimal;

#[derive(Debug, Clone, Builder, PartialEq)]
pub struct Tick {
    pub timestamp: i64,
    pub symbol: String,
    pub price: Decimal,
}

impl From<Tick> for SymbolPrice {
    fn from(value: Tick) -> Self {
        SymbolPrice::builder()
            .symbol(value.symbol)
            .price(value.price)
            .build()
    }
}
