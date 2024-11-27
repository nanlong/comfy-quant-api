use super::SymbolPriceStorable;
use anyhow::Result;
use comfy_quant_exchange::client::spot_client::base::SymbolPrice;
use rust_decimal::Decimal;
use std::collections::HashMap;

type SymbolPriceStoreMap = HashMap<String, Decimal>;

#[derive(Debug, Default)]
pub(crate) struct SymbolPriceStore {
    inner: SymbolPriceStoreMap,
}

impl AsRef<SymbolPriceStoreMap> for SymbolPriceStore {
    fn as_ref(&self) -> &SymbolPriceStoreMap {
        &self.inner
    }
}

impl AsMut<SymbolPriceStoreMap> for SymbolPriceStore {
    fn as_mut(&mut self) -> &mut SymbolPriceStoreMap {
        &mut self.inner
    }
}

impl SymbolPriceStore {
    pub fn new() -> Self {
        SymbolPriceStore {
            inner: HashMap::new(),
        }
    }

    pub fn price(&self, symbol: impl AsRef<str>) -> Option<&Decimal> {
        self.as_ref().get(symbol.as_ref())
    }
}

impl SymbolPriceStorable for SymbolPriceStore {
    fn save_price(&mut self, symbol_price: SymbolPrice) -> Result<()> {
        self.as_mut()
            .insert(symbol_price.symbol, symbol_price.price);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_tick_store() {
        let mut store = SymbolPriceStore::new();
        assert_eq!(store.price("BTCUSDT"), None);

        let price = SymbolPrice::builder()
            .symbol("BTCUSDT")
            .price(dec!(90000))
            .build();

        store.save_price(price).unwrap();
        assert_eq!(store.price("BTCUSDT"), Some(&dec!(90000)));
    }
}
