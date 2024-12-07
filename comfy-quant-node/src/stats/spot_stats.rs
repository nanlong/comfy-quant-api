use super::spot_stats_data::SpotStatsData;
use crate::node_core::{NodeContext, Tick};
use anyhow::Result;
use comfy_quant_base::ExchangeSymbolKey;
use comfy_quant_exchange::client::spot_client::base::Order;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

type SpotStatsDataMap = HashMap<ExchangeSymbolKey, SpotStatsData>;

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct SpotStats {
    data: SpotStatsDataMap,
}

impl AsRef<SpotStatsDataMap> for SpotStats {
    fn as_ref(&self) -> &SpotStatsDataMap {
        &self.data
    }
}

impl AsMut<SpotStatsDataMap> for SpotStats {
    fn as_mut(&mut self) -> &mut SpotStatsDataMap {
        &mut self.data
    }
}

impl SpotStats {
    pub fn new() -> Self {
        SpotStats {
            data: SpotStatsDataMap::new(),
        }
    }

    pub fn get(
        &self,
        exchange: impl AsRef<str>,
        symbol: impl AsRef<str>,
    ) -> Option<&SpotStatsData> {
        let key = ExchangeSymbolKey::new(exchange.as_ref(), symbol.as_ref());
        self.data.get(&key)
    }

    pub fn get_or_insert(
        &mut self,
        exchange: impl AsRef<str>,
        symbol: impl AsRef<str>,
    ) -> &mut SpotStatsData {
        let key = ExchangeSymbolKey::new(exchange.as_ref(), symbol.as_ref());
        self.as_mut().entry(key).or_default()
    }

    pub fn initialize(
        &mut self,
        exchange: impl AsRef<str>,
        symbol: impl AsRef<str>,
        base_asset: impl AsRef<str>,
        quote_asset: impl AsRef<str>,
    ) {
        self.get_or_insert(exchange.as_ref(), symbol.as_ref())
            .initialize(
                exchange.as_ref(),
                symbol.as_ref(),
                base_asset.as_ref(),
                quote_asset.as_ref(),
            );
    }

    pub async fn initialize_balance(
        &mut self,
        ctx: NodeContext,
        exchange: impl AsRef<str>,
        symbol: impl AsRef<str>,
        initial_base: &Decimal,
        initial_quote: &Decimal,
        initial_price: &Decimal,
    ) -> Result<()> {
        self.get_or_insert(exchange.as_ref(), symbol.as_ref())
            .initialize_balance(ctx, initial_base, initial_quote, initial_price)
            .await?;
        Ok(())
    }

    pub async fn update_with_tick(
        &mut self,
        ctx: NodeContext,
        exchange: impl AsRef<str>,
        symbol: impl AsRef<str>,
        tick: &Tick,
    ) -> Result<()> {
        self.get_or_insert(exchange.as_ref(), symbol.as_ref())
            .update_with_tick(ctx, tick)
            .await?;
        Ok(())
    }

    pub async fn update_with_order(
        &mut self,
        ctx: NodeContext,
        exchange: impl AsRef<str>,
        symbol: impl AsRef<str>,
        order: &Order,
    ) -> Result<()> {
        self.get_or_insert(exchange.as_ref(), symbol.as_ref())
            .update_with_order(ctx, order)
            .await?;

        Ok(())
    }
}
