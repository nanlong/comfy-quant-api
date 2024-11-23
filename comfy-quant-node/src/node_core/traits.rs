use super::spot_stats::{SpotStats, SpotStatsInner};
use crate::{node_core::Port, workflow::Node};
use anyhow::Result;
use comfy_quant_database::strategy_spot_stats::SpotStatsUniqueKey;
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
    fn get_port(&self) -> &Port;

    fn get_port_mut(&mut self) -> &mut Port;
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
    fn get_spot_stats(&self) -> Option<&SpotStats> {
        None
    }

    fn get_spot_stats_mut(&mut self) -> Option<&mut SpotStats> {
        None
    }

    fn get_spot_stats_inner(&self, key: impl AsRef<str>) -> Result<&SpotStatsInner> {
        self.get_spot_stats()
            .ok_or_else(|| anyhow::anyhow!("Spot stats not found"))?
            .get(key.as_ref())
            .ok_or_else(|| anyhow::anyhow!("Stats not found for key: {}", key.as_ref()))
    }

    fn update_spot_stats_with_order(&mut self, key: impl AsRef<str>, order: &Order) -> Result<()> {
        let stats = self
            .get_spot_stats_mut()
            .ok_or_else(|| anyhow::anyhow!("Spot stats not found"))?;

        stats.get_or_insert(key.as_ref()).update_with_order(order)?;

        Ok(())
    }
}

/// 价格接口
#[allow(async_fn_in_trait)]
pub trait NodeSymbolPrice {
    async fn get_price(&self, symbol: impl AsRef<str>) -> Option<Decimal>;
}

/// 节点名称接口
pub trait NodeInfo {
    // 获取节点
    fn node(&self) -> &Node;

    // 节点id
    fn node_id(&self) -> u32;

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
impl<T: NodeInfo + NodeStats + NodeSymbolPrice> SpotTradeable for T {
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
            if let Some(price) = self.get_price(&symbol).await {
                backtest_spot_client.save_price(price).await;
            }
        }

        // 提交交易
        let order = client.market_buy(base_asset, quote_asset, qty).await?;

        // 更新统计信息
        self.update_spot_stats_with_order(&stats_key, &order)?;

        let context = self.node().context()?;
        let cloned_db = context.cloned_db();
        let workflow_id = context.workflow_id();
        let node_id = self.node_id() as i16;
        let node_name = self.node_name();
        let exchange = client.platform_name();
        let stats = self.get_spot_stats_inner(&stats_key)?;
        let params = SpotStatsUniqueKey::builder()
            .workflow_id(workflow_id)
            .node_id(node_id)
            .node_name(node_name)
            .exchange(exchange)
            .symbol(&symbol)
            .base_asset(base_asset)
            .quote_asset(quote_asset)
            .build();

        // 保存策略仓位信息到数据库
        stats
            .save_strategy_spot_position(&cloned_db, &params)
            .await?;

        // 保存统计信息到数据库
        stats.save_strategy_spot_stats(&cloned_db, &params).await?;

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
            if let Some(price) = self.get_price(&symbol).await {
                backtest_spot_client.save_price(price).await;
            }
        }

        // 提交交易
        let order = client.market_sell(base_asset, quote_asset, qty).await?;

        // 更新统计信息
        self.update_spot_stats_with_order(&stats_key, &order)?;

        let context = self.node().context()?;
        let cloned_db = context.cloned_db();
        let workflow_id = context.workflow_id();
        let node_id = self.node_id() as i16;
        let node_name = self.node_name();
        let exchange = client.platform_name();
        let stats = self.get_spot_stats_inner(&stats_key)?;
        let params = SpotStatsUniqueKey::builder()
            .workflow_id(workflow_id)
            .node_id(node_id)
            .node_name(node_name)
            .exchange(exchange)
            .symbol(&symbol)
            .base_asset(base_asset)
            .quote_asset(quote_asset)
            .build();

        // 保存策略仓位信息到数据库
        stats
            .save_strategy_spot_position(&cloned_db, &params)
            .await?;

        // 保存统计信息到数据库
        stats.save_strategy_spot_stats(&cloned_db, &params).await?;

        Ok(order)
    }
}
