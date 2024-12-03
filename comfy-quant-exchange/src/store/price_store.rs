use crate::client::spot_client::base::{Exchange, ExchangeMarketSymbolKey, Market, SymbolPrice};
use anyhow::Result;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

type PriceStoreMap = HashMap<ExchangeMarketSymbolKey, Decimal>;

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct PriceStore {
    inner: PriceStoreMap,
}

impl AsRef<PriceStoreMap> for PriceStore {
    fn as_ref(&self) -> &PriceStoreMap {
        &self.inner
    }
}

impl AsMut<PriceStoreMap> for PriceStore {
    fn as_mut(&mut self) -> &mut PriceStoreMap {
        &mut self.inner
    }
}

impl PriceStore {
    pub fn new() -> Self {
        PriceStore {
            inner: HashMap::new(),
        }
    }

    pub fn price(
        &self,
        exchange: impl AsRef<str>,
        market: impl AsRef<str>,
        symbol: impl AsRef<str>,
    ) -> Option<Decimal> {
        let key =
            ExchangeMarketSymbolKey::try_new(exchange.as_ref(), market.as_ref(), symbol.as_ref())
                .ok()?;
        self.as_ref().get(&key).cloned()
    }

    pub fn save_price(
        &mut self,
        exchange: Exchange,
        market: Market,
        symbol_price: SymbolPrice,
    ) -> Result<()> {
        let key = ExchangeMarketSymbolKey::try_new(exchange, market, symbol_price.symbol)?;
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
        let market = Market::Spot;
        let mut store = PriceStore::new();
        assert_eq!(store.price(&exchange, &market, "BTCUSDT"), None);

        let price = SymbolPrice::builder()
            .symbol("BTCUSDT")
            .price(dec!(90000))
            .build();

        store
            .save_price(exchange.clone(), market.clone(), price)
            .unwrap();
        assert_eq!(
            store.price(&exchange, &market, "BTCUSDT"),
            Some(dec!(90000))
        );
    }
}
