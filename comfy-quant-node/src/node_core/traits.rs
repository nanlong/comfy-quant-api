use super::stats::Stats;
use crate::{node_core::Port, workflow::WorkflowContext};
use anyhow::Result;
use comfy_quant_exchange::client::{
    spot_client::base::{Order, SymbolPrice},
    spot_client_kind::{SpotClientExecutable, SpotClientKind},
};
use enum_dispatch::enum_dispatch;
use rust_decimal::Decimal;
use std::sync::Arc;

/// 节点初始化
#[enum_dispatch]
pub trait Setupable {
    fn setup_context(&mut self, context: Arc<WorkflowContext>);

    fn get_context(&self) -> Result<&Arc<WorkflowContext>>;
}

// 节点执行
#[enum_dispatch]
#[allow(async_fn_in_trait)]
pub trait Executable {
    async fn execute(&mut self) -> Result<()>;
}

// 节点插槽
#[enum_dispatch]
pub trait PortAccessor {
    fn get_port(&self) -> &Port;

    fn get_port_mut(&mut self) -> &mut Port;
}

// 节点连接
#[enum_dispatch]
pub trait Connectable {
    fn connection<U: Send + Sync + 'static>(
        &self,                         // 当前节点
        target: &mut dyn PortAccessor, // 目标节点
        origin_slot: usize,            // 当前节点输出槽位
        target_slot: usize,            // 目标节点输入槽位
    ) -> Result<()>;
}

// 节点连接默认实现
impl<T: PortAccessor> Connectable for T {
    fn connection<U: Send + Sync + 'static>(
        &self,                         // 当前节点
        target: &mut dyn PortAccessor, // 目标节点
        origin_slot: usize,            // 当前节点输出槽位
        target_slot: usize,            // 目标节点输入槽位
    ) -> Result<()> {
        let slot = self.get_port().get_output::<U>(origin_slot)?;
        target.get_port_mut().add_input(target_slot, slot)?;

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
    fn get_stats(&self) -> &Stats;

    fn get_stats_mut(&mut self) -> &mut Stats;

    fn update_stats_with_order(&mut self, order: &Order) -> Result<()> {
        self.get_stats_mut().update_with_order(order)
    }
}

/// 价格接口
#[allow(async_fn_in_trait)]
pub trait NodeSymbolPrice {
    async fn get_price(&self, symbol: &str) -> Option<Decimal>;
}

/// 节点名称接口
pub trait NodeName {
    fn get_name(&self) -> &str;
}

/// 策略统计信息接口
/// 需要从context中获取到db
pub trait StrategyStats: Setupable {
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
impl<T: Setupable + NodeStats + NodeSymbolPrice + NodeName> SpotTradeable for T {
    async fn market_buy(
        &mut self,
        client: &SpotClientKind,
        base_asset: &str,
        quote_asset: &str,
        qty: f64,
    ) -> Result<Order> {
        // 用于回测功能的客户端，需要知道当前价格
        let symbol = client.symbol(base_asset, quote_asset);

        if let SpotClientKind::BacktestSpotClient(backtest_spot_client) = client {
            if let Some(price) = self.get_price(&symbol).await {
                backtest_spot_client.save_price(price).await;
            }
        }

        // 提交交易
        let order = client.market_buy(base_asset, quote_asset, qty).await?;

        // 更新统计信息
        self.update_stats_with_order(&order)?;

        // 更新数据库
        let _cloned_db = self.get_context()?.cloned_db();
        let _workflow_id = self.get_context()?.workflow_id();
        let _node_name = self.get_name();
        let _stats = self.get_stats();

        // save to db
        // save_stats(&cloned_db, workflow_id, &stats)?;

        Ok(order)
    }

    async fn market_sell(
        &mut self,
        client: &SpotClientKind,
        base_asset: &str,
        quote_asset: &str,
        qty: f64,
    ) -> Result<Order> {
        // 用于回测功能的客户端，需要知道当前价格
        let symbol = client.symbol(base_asset, quote_asset);

        if let SpotClientKind::BacktestSpotClient(backtest_spot_client) = client {
            if let Some(price) = self.get_price(&symbol).await {
                backtest_spot_client.save_price(price).await;
            }
        }

        // 提交交易
        let order = client.market_sell(base_asset, quote_asset, qty).await?;

        // 更新统计信息
        self.update_stats_with_order(&order)?;

        // 更新数据库
        let _cloned_db = self.get_context()?.cloned_db();
        let _workflow_id = self.get_context()?.workflow_id();
        let _node_name = self.get_name();
        let _stats = self.get_stats();

        Ok(order)
    }
}
