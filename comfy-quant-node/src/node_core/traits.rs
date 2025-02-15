use super::{NodeContext, NodeInfra, Tick};
use crate::{
    node_core::Port,
    stats::{SpotStats, SpotStatsData},
    workflow::{Node, WorkflowContext},
};
use anyhow::Result;
use comfy_quant_base::{Exchange, Market, Symbol};
// use chrono::{DateTime, Utc};
// use comfy_quant_base::KlineInterval;
use comfy_quant_exchange::client::{
    spot_client::base::Order,
    spot_client_kind::{SpotClientExecutable, SpotClientKind, SpotclientExecutableExt},
};
use enum_dispatch::enum_dispatch;
use rust_decimal::{Decimal, MathematicalOps};
use std::sync::Arc;

#[enum_dispatch]
pub trait NodeCore {
    fn node_infra(&self) -> &NodeInfra;

    fn node_infra_mut(&mut self) -> &mut NodeInfra;
}

impl<T: ?Sized> NodeCoreExt for T where T: NodeCore {}

#[allow(async_fn_in_trait)]
pub trait NodeCoreExt: NodeCore {
    fn node(&self) -> &Node {
        self.node_infra().node()
    }

    fn port(&self) -> &Port {
        self.node_infra().port()
    }

    fn port_mut(&mut self) -> &mut Port {
        self.node_infra_mut().port_mut()
    }

    fn workflow_context(&self) -> Result<&Arc<WorkflowContext>> {
        self.node_infra().workflow_context()
    }

    fn node_context(&self) -> Result<NodeContext> {
        self.node_infra().node_context()
    }

    fn connection<U: Send + Sync + 'static>(
        &self,                     // 当前节点
        target: &mut dyn NodeCore, // 目标节点
        origin_slot: usize,        // 当前节点输出槽位
        target_slot: usize,        // 目标节点输入槽位
    ) -> Result<()> {
        let slot = self.port().output::<U>(origin_slot)?;
        target.port_mut().set_input(target_slot, slot)?;

        Ok(())
    }

    async fn price(
        &self,
        exchange: &Exchange,
        market: &Market,
        symbol: &Symbol,
    ) -> Result<Decimal> {
        self.node_infra().price(exchange, market, symbol).await
    }
}

pub trait NodeSpotStats {
    fn spot_stats(&self) -> &SpotStats;

    fn spot_stats_mut(&mut self) -> &mut SpotStats;
}

impl<T: ?Sized> NodeSpotStatsExt for T where T: NodeCore + NodeSpotStats {}

#[allow(async_fn_in_trait)]
pub trait NodeSpotStatsExt: NodeCore + NodeSpotStats {
    fn spot_stats_data(&self, exchange: &Exchange, symbol: &Symbol) -> Result<&SpotStatsData> {
        self.spot_stats().get(exchange, symbol).ok_or_else(|| {
            anyhow::anyhow!(
                "Stats not found for exchange: {} symbol: {}",
                exchange,
                symbol
            )
        })
    }

    async fn update_spot_stats_with_tick(
        &mut self,
        exchange: &Exchange,
        symbol: &Symbol,
        tick: &Tick,
    ) -> Result<()> {
        let ctx = self.node_context()?;

        self.spot_stats_mut()
            .update_with_tick(&ctx, exchange, symbol, tick)
            .await?;

        Ok(())
    }

    async fn update_spot_stats_with_order(
        &mut self,
        exchange: &Exchange,
        symbol: &Symbol,
        order: &Order,
    ) -> Result<()> {
        let ctx = self.node_context()?;

        self.spot_stats_mut()
            .update_with_order(&ctx, exchange, symbol, order)
            .await?;

        Ok(())
    }
}

impl<T: ?Sized> SpotTradeable for T where T: NodeCore + NodeSpotStats {}

/// 交易接口
#[allow(async_fn_in_trait)]
pub trait SpotTradeable: NodeCore + NodeSpotStats {
    async fn market_buy(
        &mut self,
        client: &SpotClientKind,
        base_asset: &str,
        quote_asset: &str,
        qty: f64,
    ) -> Result<Order> {
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

// 节点执行
#[enum_dispatch]
#[allow(async_fn_in_trait)]
pub trait NodeExecutable {
    async fn setup(&mut self) -> Result<()> {
        Ok(())
    }

    async fn execute(&mut self) -> Result<()> {
        Ok(())
    }
}

// pub struct DateTimeRange {
//     start: DateTime<Utc>,
//     end: DateTime<Utc>,
// }

// impl DateTimeRange {
//     pub fn new(start: DateTime<Utc>, end: DateTime<Utc>) -> Self {
//         Self { start, end }
//     }
// }

// pub struct AssetPoint {
//     time: DateTime<Utc>,
//     value: Decimal,
// }

// impl AssetPoint {
//     pub fn new(time: DateTime<Utc>, value: Decimal) -> Self {
//         Self { time, value }
//     }
// }

// 基础交易统计trait
#[allow(async_fn_in_trait)]
pub trait TradeStats {
    // 初始资金
    async fn initial_capital(&self) -> Result<Decimal>;
    // 已实现盈亏
    async fn realized_pnl(&self) -> Result<Decimal>;
    // 未实现盈亏
    async fn unrealized_pnl(&self) -> Result<Decimal>;
    // 运行时间
    async fn running_time(&self) -> Result<u128>;
    // 资产历史
    // async fn asset_history(
    //     &self,
    //     interval: KlineInterval,
    //     range: DateTimeRange,
    // ) -> Result<Vec<AssetPoint>>;
    // // 最大回撤
    // fn max_drawdown(&self) -> Decimal;
    // // 夏普比率
    // fn sharpe_ratio(&self) -> Decimal;
    // // 收益率曲线
    // fn return_chart(&self) -> Vec<(i64, Decimal)>;
    // // 资金曲线
    // fn equity_curve(&self) -> Vec<(i64, Decimal)>;
}

impl<T: ?Sized> TradeStatsExt for T where T: TradeStats {}

#[allow(unused)]
pub trait TradeStatsExt: TradeStats {
    // 总盈亏
    async fn total_pnl(&self) -> Result<Decimal> {
        let realized_pnl = self.realized_pnl().await?;
        let unrealized_pnl = self.unrealized_pnl().await?;
        Ok(realized_pnl + unrealized_pnl)
    }

    // 总收益率
    async fn total_return(&self) -> Result<Decimal> {
        let initial_capital = self.initial_capital().await?;
        let realized_pnl = self.realized_pnl().await?;
        let unrealized_pnl = self.unrealized_pnl().await?;

        // 防止除以零
        if initial_capital.is_zero() {
            return Ok(Decimal::ZERO);
        }

        // 收益率 = (已实现盈亏 + 未实现盈亏) / 初始资金
        let return_rate = (realized_pnl + unrealized_pnl) / initial_capital;

        Ok(return_rate)
    }

    // 运行天数
    async fn running_days(&self) -> Result<Decimal> {
        let running_time = self.running_time().await?;
        let running_days = Decimal::from(running_time / 1_000_000 / 86_400);
        Ok(running_days)
    }

    // 年化收益率
    async fn annualized_return(&self) -> Result<Decimal> {
        let total_return = self.total_return().await?;
        let running_days = self.running_days().await?;

        // 如果运行时间太短，返回0
        if running_days < Decimal::new(1, 0) {
            return Ok(Decimal::ZERO);
        }

        // 年化收益率 = (1 + r)^(365/t) - 1
        // 其中 r 是总收益率(小数形式), t 是运行天数
        let base = Decimal::ONE + total_return;
        let power = Decimal::from(365) / running_days;

        // 使用自然对数计算幂
        let result = base.ln() * power;
        let annualized = result.exp() - Decimal::ONE;

        Ok(annualized)
    }
}
