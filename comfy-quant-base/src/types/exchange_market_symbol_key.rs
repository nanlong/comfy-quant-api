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
        market: impl TryInto<Market>,
        symbol: impl Into<Symbol>,
    ) -> Result<Self> {
        let market = market
            .try_into()
            .map_err(|_| anyhow::anyhow!("Invalid market"))?;

        Ok(Self {
            exchange: exchange.into(),
            market,
            symbol: symbol.into(),
        })
    }
}
