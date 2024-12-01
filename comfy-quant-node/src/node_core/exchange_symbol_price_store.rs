use super::{ExchangeSymbolKey, SymbolPriceStorable};
use anyhow::Result;
use comfy_quant_exchange::client::spot_client::base::{Exchange, SymbolPrice};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

type ExchangeSymbolPriceStoreMap = HashMap<ExchangeSymbolKey, Decimal>;

#[derive(Serialize, Deserialize, Debug, Default)]
pub(crate) struct ExchangeSymbolPriceStore {
    inner: ExchangeSymbolPriceStoreMap,
}

impl AsRef<ExchangeSymbolPriceStoreMap> for ExchangeSymbolPriceStore {
    fn as_ref(&self) -> &ExchangeSymbolPriceStoreMap {
        &self.inner
    }
}

impl AsMut<ExchangeSymbolPriceStoreMap> for ExchangeSymbolPriceStore {
    fn as_mut(&mut self) -> &mut ExchangeSymbolPriceStoreMap {
        &mut self.inner
    }
}

impl ExchangeSymbolPriceStore {
    pub fn new() -> Self {
        ExchangeSymbolPriceStore {
            inner: HashMap::new(),
        }
    }

    pub fn price(&self, exchange: impl AsRef<str>, symbol: impl AsRef<str>) -> Option<&Decimal> {
        let key = ExchangeSymbolKey::new(exchange.as_ref(), symbol.as_ref());
        self.as_ref().get(&key)
    }
}

impl SymbolPriceStorable for ExchangeSymbolPriceStore {
    fn save_price(&mut self, exchange: Exchange, symbol_price: SymbolPrice) -> Result<()> {
        let key = ExchangeSymbolKey::new(exchange, symbol_price.symbol);
        self.as_mut().insert(key, symbol_price.price);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_tick_store() {
        let exchange = Exchange::new("Binance");
        let mut store = ExchangeSymbolPriceStore::new();
        assert_eq!(store.price(&exchange, "BTCUSDT"), None);

        let price = SymbolPrice::builder()
            .symbol("BTCUSDT")
            .price(dec!(90000))
            .build();

        store.save_price(exchange.clone(), price).unwrap();
        assert_eq!(store.price(&exchange, "BTCUSDT"), Some(&dec!(90000)));
    }
}
