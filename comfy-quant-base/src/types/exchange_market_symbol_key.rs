use super::{Exchange, Market, Symbol};
use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Hash, Eq, PartialEq)]
pub struct ExchangeMarketSymbolKey {
    pub exchange: Exchange,
    pub market: Market,
    pub symbol: Symbol,
}

impl ExchangeMarketSymbolKey {
    pub fn try_new(
        exchange: impl Into<Exchange>,
        market: impl Into<Market>,
        symbol: impl Into<Symbol>,
    ) -> Result<Self> {
        Ok(Self {
            exchange: exchange.into(),
            market: market.into(),
            symbol: symbol.into(),
        })
    }
}
