use super::spot_stats::{SpotStats, SpotStatsData};
use crate::{node_core::Port, workflow::Node};
use anyhow::Result;
use comfy_quant_exchange::client::{
    spot_client::base::{Order, SymbolPrice},
    spot_client_kind::{SpotClientExecutable, SpotClientKind},
};
use enum_dispatch::enum_dispatch;
use rust_decimal::Decimal;

// 节点执行
#[enum_dispatch]
#[allow(async_fn_in_trait)]
pub trait NodeExecutable {
    async fn execute(&mut self) -> Result<()>;
}

// 节点插槽
#[enum_dispatch]
pub trait NodePort {
    fn port(&self) -> &Port;

    fn port_mut(&mut self) -> &mut Port;
}

// 节点连接
#[enum_dispatch]
pub trait Connectable {
    fn connection<U: Send + Sync + 'static>(
        &self,                     // 当前节点
        target: &mut dyn NodePort, // 目标节点
        origin_slot: usize,        // 当前节点输出槽位
        target_slot: usize,        // 目标节点输入槽位
    ) -> Result<()>;
}

// 节点连接默认实现
impl<T: NodePort> Connectable for T {
    fn connection<U: Send + Sync + 'static>(
        &self,                     // 当前节点
        target: &mut dyn NodePort, // 目标节点
        origin_slot: usize,        // 当前节点输出槽位
        target_slot: usize,        // 目标节点输入槽位
    ) -> Result<()> {
        let slot = self.port().output::<U>(origin_slot)?;
        target.port_mut().set_input(target_slot, slot)?;

        Ok(())
    }
}

/// 价格存储介质
pub(crate) trait SymbolPriceStorable: Send + Sync + 'static {
    fn save_price(&mut self, symbol_price: SymbolPrice) -> Result<()>;
}

/// 统计接口
#[allow(async_fn_in_trait)]
pub trait NodeStats {
    fn spot_stats(&self) -> Option<&SpotStats> {
        None
    }

    fn spot_stats_mut(&mut self) -> Option<&mut SpotStats> {
        None
    }

    fn spot_stats_data(&self, key: impl AsRef<str>) -> Result<&SpotStatsData> {
        self.spot_stats()
            .ok_or_else(|| anyhow::anyhow!("Spot stats not found"))?
            .data()
            .get(key.as_ref())
            .ok_or_else(|| anyhow::anyhow!("Stats not found for key: {}", key.as_ref()))
    }

    async fn update_spot_stats_with_order(
        &mut self,
        key: impl AsRef<str>,
        order: &Order,
    ) -> Result<()> {
        let stats = self
            .spot_stats_mut()
            .ok_or_else(|| anyhow::anyhow!("Spot stats not found"))?;

        stats.update_with_order(key.as_ref(), order).await?;

        Ok(())
    }
}

/// 价格接口
#[allow(async_fn_in_trait)]
pub trait NodeSymbolPrice {
    async fn price(&self, symbol: impl AsRef<str>) -> Option<Decimal>;
}

/// 节点名称接口
pub trait NodeInfo {
    // 获取节点
    fn node(&self) -> &Node;

    // 节点id
    fn node_id(&self) -> i16;

    // 节点名称
    fn node_name(&self) -> &str;
}

/// 策略统计信息接口
/// 需要从context中获取到db
pub trait NodeStatsInfo: NodeInfo {
    // 最大回撤
    fn max_drawdown(&self, start_timestamp: i64, end_timestamp: i64) -> Decimal;
}

/// 交易接口
#[allow(async_fn_in_trait)]
pub trait SpotTradeable {
    async fn market_buy(
        &mut self,
        client: &SpotClientKind,
        base_asset: &str,
        quote_asset: &str,
        qty: f64,
    ) -> Result<Order>;

    async fn market_sell(
        &mut self,
        client: &SpotClientKind,
        base_asset: &str,
        quote_asset: &str,
        qty: f64,
    ) -> Result<Order>;
}

/// 交易接口默认实现
impl<T: NodeStats + NodeSymbolPrice> SpotTradeable for T {
    async fn market_buy(
        &mut self,
        client: &SpotClientKind,
        base_asset: &str,
        quote_asset: &str,
        qty: f64,
    ) -> Result<Order> {
        // 用于回测功能的客户端，需要知道当前价格
        let symbol = client.symbol(base_asset, quote_asset);
        let stats_key = client.stats_key(&symbol);

        if let SpotClientKind::BacktestSpotClient(backtest_spot_client) = client {
            if let Some(price) = self.price(&symbol).await {
                backtest_spot_client.save_price(price).await;
            }
        }

        // 提交交易
        let order = client.market_buy(base_asset, quote_asset, qty).await?;

        // 更新统计信息
        self.update_spot_stats_with_order(&stats_key, &order)
            .await?;

        Ok(order)
    }

    async fn market_sell(
        &mut self,
        client: &SpotClientKind,
        base_asset: &str,
        quote_asset: &str,
        qty: f64,
    ) -> Result<Order> {
        let symbol = client.symbol(base_asset, quote_asset);
        let stats_key = client.stats_key(&symbol);

        // 用于回测功能的客户端，需要知道当前价格
        if let SpotClientKind::BacktestSpotClient(backtest_spot_client) = client {
            if let Some(price) = self.price(&symbol).await {
                backtest_spot_client.save_price(price).await;
            }
        }

        // 提交交易
        let order = client.market_sell(base_asset, quote_asset, qty).await?;

        // 更新统计信息
        self.update_spot_stats_with_order(&stats_key, &order)
            .await?;

        Ok(order)
    }
}
