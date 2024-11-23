use super::SymbolPriceStorable;
use anyhow::Result;
use comfy_quant_exchange::client::spot_client::base::SymbolPrice;
use rust_decimal::Decimal;
use std::{
    collections::HashMap,
    ops::{Deref, DerefMut},
};

#[derive(Debug, Default)]
pub(crate) struct SymbolPriceStore {
    inner: HashMap<String, Decimal>,
}

impl Deref for SymbolPriceStore {
    type Target = HashMap<String, Decimal>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for SymbolPriceStore {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl SymbolPriceStore {
    pub fn price(&self, symbol: impl AsRef<str>) -> Option<&Decimal> {
        self.get(symbol.as_ref())
    }
}

impl SymbolPriceStorable for SymbolPriceStore {
    fn save_price(&mut self, symbol_price: SymbolPrice) -> Result<()> {
        self.insert(symbol_price.symbol, symbol_price.price);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_tick_store() {
        let mut store = SymbolPriceStore::default();
        assert_eq!(store.price("BTCUSDT"), None);

        let price = SymbolPrice::builder()
            .symbol("BTCUSDT")
            .price(dec!(90000))
            .build();

        store.save_price(price).unwrap();
        assert_eq!(store.price("BTCUSDT"), Some(&dec!(90000)));
    }
}
