use super::{NodeContext, Tick};
use crate::{
    node_core::Port,
    stats::{SpotStats, SpotStatsData},
};
use anyhow::Result;
// use comfy_quant_base::{secs_to_datetime, Market};
// use comfy_quant_database::{kline, strategy_spot_position};
use comfy_quant_exchange::client::{
    spot_client::base::Order,
    spot_client_kind::{SpotClientExecutable, SpotClientKind},
};
use enum_dispatch::enum_dispatch;
use rust_decimal::Decimal;
// use rust_decimal::Decimal;

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

/// 节点名称接口
pub trait NodeInfo {
    // 获取节点
    fn node_context(&self) -> Result<NodeContext>;
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

    async fn update_spot_stats_with_tick(
        &mut self,
        exchange: impl AsRef<str>,
        symbol: impl AsRef<str>,
        tick: &Tick,
    ) -> Result<()> {
        let ctx = self.node_context()?;

        let stats = self
            .spot_stats_mut()
            .ok_or_else(|| anyhow::anyhow!("Spot stats not found"))?;

        stats.update_with_tick(ctx, exchange, symbol, tick).await?;

        Ok(())
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

pub struct AssetAmount {
    asset: String,
    value: Decimal,
}

impl AssetAmount {
    pub fn new(asset: impl Into<String>, value: Decimal) -> Self {
        Self {
            asset: asset.into(),
            value,
        }
    }

    pub fn asset(&self) -> &str {
        &self.asset
    }

    pub fn value(&self) -> &Decimal {
        &self.value
    }
}

// 基础交易统计trait
#[allow(async_fn_in_trait)]
pub trait TradeStats {
    // 初始资金
    async fn initial_capital(&self) -> Result<AssetAmount>;
    // 已实现盈亏
    async fn realized_pnl(&self) -> Result<AssetAmount>;
    // 未实现盈亏
    async fn unrealized_pnl(&self) -> Result<AssetAmount>;
    // 总盈亏
    async fn total_pnl(&self) -> Result<AssetAmount>;
    // // 收益率
    // fn total_return(&self) -> Decimal;
    // // 年化收益率
    // fn annualized_return(&self) -> Decimal;
    // // 最大回撤
    // fn max_drawdown(&self) -> Decimal;
    // // 夏普比率
    // fn sharpe_ratio(&self) -> Decimal;
    // // 收益率曲线
    // fn return_chart(&self) -> Vec<(i64, Decimal)>;
    // // 资金曲线
    // fn equity_curve(&self) -> Vec<(i64, Decimal)>;
}

// impl<T: NodeExecutable> TradeStats for T {
//     fn realized_pnl(&self) -> AssetAmount {
//         AssetAmount::new("USDT", Decimal::ZERO)
//     }
// }
