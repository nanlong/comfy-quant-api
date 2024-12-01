use comfy_quant_exchange::client::spot_client::base::{Exchange, Symbol};
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
