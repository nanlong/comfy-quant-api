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

    pub fn setup(
        &mut self,
        exchange: impl AsRef<str>,
        symbol: impl AsRef<str>,
        base_asset: impl AsRef<str>,
        quote_asset: impl AsRef<str>,
    ) {
        self.get_or_insert(exchange.as_ref(), symbol.as_ref())
            .setup(
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

#[cfg(test)]
mod tests {
    use super::*;
    use comfy_quant_exchange::client::spot_client::base::{
        Order, OrderSide, OrderStatus, OrderType,
    };
    use rust_decimal_macros::dec;
    use sqlx::PgPool;
    use std::{str::FromStr, sync::Arc};

    fn create_test_order(side: OrderSide, price: &str, quantity: &str) -> Order {
        Order::builder()
            .order_id("test_order")
            .client_order_id("test_client_order")
            .symbol("BTC/USDT")
            .order_side(side)
            .order_status(OrderStatus::Filled)
            .price(price)
            .orig_qty(quantity)
            .executed_qty(quantity)
            .cumulative_quote_qty(
                (Decimal::from_str(price).unwrap() * Decimal::from_str(quantity).unwrap())
                    .to_string(),
            )
            .avg_price(price)
            .exchange("test")
            .base_asset("BTC")
            .quote_asset("USDT")
            .order_type(OrderType::Limit)
            .time(0)
            .update_time(0)
            .build()
    }

    #[test]
    fn test_spot_stats_data_setup() {
        let mut data = SpotStatsData::new();
        data.setup("binance", "BTC/USDT", "BTC", "USDT");

        assert_eq!(data.base.exchange, "binance");
        assert_eq!(data.base.symbol, "BTC/USDT");
        assert_eq!(data.base.base_asset, "BTC");
        assert_eq!(data.base.quote_asset, "USDT");
    }

    #[sqlx::test(migrator = "comfy_quant_database::MIGRATOR")]
    async fn test_spot_stats_data_update_with_buy_order(db: PgPool) {
        let mut data = SpotStatsData::new();
        data.setup("binance", "BTC/USDT", "BTC", "USDT");
        data.base.maker_commission_rate = dec!(0.001);
        data.quote_asset_balance = dec!(10000);

        let order = create_test_order(OrderSide::Buy, "50000", "0.1");

        // 模拟数据库连接和上下文
        let db = Arc::new(db);
        let workflow_id = "test_workflow";
        let node_id = 1_i16;
        let node_name = "test_node";
        let ctx = NodeContext::new(db, workflow_id, node_id, node_name);

        // 更新订单信息
        let result = data.update_with_order(ctx, &order).await;
        assert!(result.is_ok());

        // 验证数据更新
        assert_eq!(data.base.total_trades, 1);
        assert_eq!(data.base.buy_trades, 1);
        assert_eq!(data.base.sell_trades, 0);
        assert_eq!(data.base_asset_balance, dec!(0.0999)); // 0.1 - 0.1 * 0.001
        assert_eq!(data.quote_asset_balance, dec!(5000)); // 10000 - 50000 * 0.1
        assert_eq!(data.avg_price, dec!(50000));
    }

    #[sqlx::test(migrator = "comfy_quant_database::MIGRATOR")]
    async fn test_spot_stats_data_update_with_sell_order(db: PgPool) {
        let mut data = SpotStatsData::new();
        data.setup("binance", "BTC/USDT", "BTC", "USDT");
        data.base.maker_commission_rate = dec!(0.001);
        data.base_asset_balance = dec!(1.0);
        data.avg_price = dec!(45000);

        let order = create_test_order(OrderSide::Sell, "50000", "0.1");

        // 模拟数据库连接和上下文
        let db = Arc::new(db);
        let workflow_id = "test_workflow";
        let node_id = 1_i16;
        let node_name = "test_node";
        let ctx = NodeContext::new(db, workflow_id, node_id, node_name);

        // 更新订单信息
        let result = data.update_with_order(ctx, &order).await;
        assert!(result.is_ok());

        // 验证数据更新
        assert_eq!(data.base.total_trades, 1);
        assert_eq!(data.base.buy_trades, 0);
        assert_eq!(data.base.sell_trades, 1);
        assert_eq!(data.base_asset_balance, dec!(0.9));
        assert_eq!(data.quote_asset_balance, dec!(4995)); // 5000 * 0.999

        // 验证盈亏计算
        let expected_pnl = dec!(4995) - dec!(0.1) * dec!(45000);
        assert_eq!(data.base.realized_pnl, expected_pnl);
        assert_eq!(data.base.win_trades, 1);
    }

    #[test]
    fn test_spot_stats_get_or_insert() {
        let exchange = "Binance";
        let symbol = "BTC/USDT";

        let mut stats = SpotStats::new();

        let data = stats.get_or_insert(exchange, symbol);
        assert_eq!(data.base.total_trades, 0);
        assert_eq!(data.base.buy_trades, 0);
        assert_eq!(data.base.sell_trades, 0);

        // 测试重复获取相同的key
        let data2 = stats.get_or_insert(exchange, symbol);
        assert_eq!(data2.base.total_trades, 0);
    }
}
