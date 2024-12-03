use anyhow::Result;
use comfy_quant_exchange::client::spot_client::base::{Exchange, Market, Symbol};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Hash, Eq, PartialEq)]
pub struct ExchangeSymbolKey {
    pub exchange: Exchange,
    pub symbol: Symbol,
}

impl ExchangeSymbolKey {
    pub fn new(exchange: impl Into<Exchange>, symbol: impl Into<Symbol>) -> Self {
        Self {
            exchange: exchange.into(),
            symbol: symbol.into(),
        }
    }
}

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
