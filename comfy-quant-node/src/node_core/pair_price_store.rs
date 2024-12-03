use super::{ExchangeMarketSymbolKey, SymbolPriceStorable};
use anyhow::Result;
use comfy_quant_exchange::client::spot_client::base::{Exchange, Market, SymbolPrice};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

type PairPriceStoreMap = HashMap<ExchangeMarketSymbolKey, Decimal>;

#[derive(Serialize, Deserialize, Debug, Default)]
pub(crate) struct PairPriceStore {
    inner: PairPriceStoreMap,
}

impl AsRef<PairPriceStoreMap> for PairPriceStore {
    fn as_ref(&self) -> &PairPriceStoreMap {
        &self.inner
    }
}

impl AsMut<PairPriceStoreMap> for PairPriceStore {
    fn as_mut(&mut self) -> &mut PairPriceStoreMap {
        &mut self.inner
    }
}

impl PairPriceStore {
    pub fn new() -> Self {
        PairPriceStore {
            inner: HashMap::new(),
        }
    }

    pub fn price(
        &self,
        exchange: impl AsRef<str>,
        market: impl AsRef<str>,
        symbol: impl AsRef<str>,
    ) -> Option<&Decimal> {
        let key =
            ExchangeMarketSymbolKey::try_new(exchange.as_ref(), market.as_ref(), symbol.as_ref())
                .ok()?;
        self.as_ref().get(&key)
    }
}

impl SymbolPriceStorable for PairPriceStore {
    fn save_price(
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
        let mut store = PairPriceStore::new();
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
            Some(&dec!(90000))
        );
    }
}
