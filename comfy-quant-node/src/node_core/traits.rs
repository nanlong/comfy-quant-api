use super::{
    spot_stats::{SpotStats, SpotStatsData},
    NodeContext,
};
use crate::node_core::Port;
use anyhow::Result;
use comfy_quant_database::{kline, strategy_spot_position};
use comfy_quant_exchange::client::{
    spot_client::base::Order,
    spot_client_kind::{SpotClientExecutable, SpotClientKind},
};
use comfy_quant_util::secs_to_datetime;
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

/// 统计接口
#[allow(async_fn_in_trait)]
pub trait NodeStats: NodeInfo {
    fn spot_stats(&self) -> Option<&SpotStats> {
        None
    }

    fn spot_stats_mut(&mut self) -> Option<&mut SpotStats> {
        None
    }

    fn spot_stats_data(
        &self,
        exchange: impl AsRef<str>,
        symbol: impl AsRef<str>,
    ) -> Result<&SpotStatsData> {
        self.spot_stats()
            .and_then(|stats| stats.get(exchange.as_ref(), symbol.as_ref()))
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "Stats not found for exchange: {} symbol: {}",
                    exchange.as_ref(),
                    symbol.as_ref()
                )
            })
    }

    async fn update_spot_stats_with_order(
        &mut self,
        exchange: impl AsRef<str>,
        symbol: impl AsRef<str>,
        order: &Order,
    ) -> Result<()> {
        let ctx = self.node_context()?;

        let stats = self
            .spot_stats_mut()
            .ok_or_else(|| anyhow::anyhow!("Spot stats not found"))?;

        stats
            .update_with_order(ctx, exchange, symbol, order)
            .await?;

        Ok(())
    }
}

/// 节点名称接口
pub trait NodeInfo {
    // 获取节点
    fn node_context(&self) -> Result<NodeContext>;
}

/// 策略统计信息接口
/// 需要从context中获取到db
#[allow(async_fn_in_trait)]
pub trait NodeStatsInfo {
    // 最大回撤
    async fn spot_max_drawdown(
        &self,
        exchange: &str,
        symbol: &str,
        interval: &str,
        start_timestamp: i64,
        end_timestamp: i64,
    ) -> Option<Decimal>;
}

impl<T: NodeInfo + NodeStats> NodeStatsInfo for T {
    async fn spot_max_drawdown(
        &self,
        exchange: &str,
        symbol: &str,
        interval: &str,
        start_timestamp: i64,
        end_timestamp: i64,
    ) -> Option<Decimal> {
        let stats = self.spot_stats()?;
        let ctx = self.node_context().ok()?;
        let market = "spot";
        let stats_data = stats.get(exchange, symbol)?;
        let start_datetime = secs_to_datetime(start_timestamp).ok()?;
        let end_datetime = secs_to_datetime(end_timestamp).ok()?;

        let positions = strategy_spot_position::list(
            &ctx.db,
            &ctx.workflow_id,
            ctx.node_id,
            exchange,
            symbol,
            &start_datetime,
            &end_datetime,
        )
        .await
        .ok()?;

        let klines = kline::list(
            &ctx.db,
            exchange,
            market,
            symbol,
            interval,
            &start_datetime,
            &end_datetime,
        )
        .await
        .ok()?;

        let net_values = stats_data.calculate_net_value(&positions, &klines).ok()?;

        Some(SpotStatsData::get_max_drawdown(&net_values))
    }
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
impl<T: NodeStats> SpotTradeable for T {
    async fn market_buy(
        &mut self,
        client: &SpotClientKind,
        base_asset: &str,
        quote_asset: &str,
        qty: f64,
    ) -> Result<Order> {
        // 用于回测功能的客户端，需要知道当前价格
        let exchange = client.exchange();
        let symbol = client.symbol(base_asset, quote_asset);

        // 提交交易
        let order = client.market_buy(base_asset, quote_asset, qty).await?;

        // 更新统计信息
        self.update_spot_stats_with_order(&exchange, &symbol, &order)
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
        let exchange = client.exchange();
        let symbol = client.symbol(base_asset, quote_asset);

        // 提交交易
        let order = client.market_sell(base_asset, quote_asset, qty).await?;

        // 更新统计信息
        self.update_spot_stats_with_order(&exchange, &symbol, &order)
            .await?;

        Ok(order)
    }
}
