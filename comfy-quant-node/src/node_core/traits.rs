use crate::{node_core::Port, workflow::WorkflowContext};
use anyhow::Result;
use async_lock::Mutex;
use comfy_quant_exchange::client::{
    spot_client::base::Order,
    spot_client_kind::{SpotClientExecutable, SpotClientKind},
};
use enum_dispatch::enum_dispatch;
use std::sync::Arc;

use super::stats::Stats;

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
    fn get_port(&self) -> &Mutex<Port>;
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
        let slot = self
            .get_port()
            .lock_blocking()
            .get_output::<U>(origin_slot)?;

        target
            .get_port()
            .lock_blocking()
            .add_input(target_slot, slot)?;

        Ok(())
    }
}

#[allow(async_fn_in_trait)]
pub trait SpotTradeable {
    async fn market_buy(
        &self,
        client: &SpotClientKind,
        base_asset: &str,
        quote_asset: &str,
        qty: f64,
    ) -> Result<Order>;

    async fn market_sell(
        &self,
        client: &SpotClientKind,
        base_asset: &str,
        quote_asset: &str,
        qty: f64,
    ) -> Result<Order>;
}

pub trait NodeStats {
    fn get_stats(&self) -> &Mutex<Stats>;

    fn update_stats_with_order(&self, order: &Order) -> Result<()> {
        self.get_stats().lock_blocking().update_with_order(order)
    }
}

impl<T: Setupable + NodeStats> SpotTradeable for T {
    async fn market_buy(
        &self,
        client: &SpotClientKind,
        base_asset: &str,
        quote_asset: &str,
        qty: f64,
    ) -> Result<Order> {
        // 提交交易
        let order = client.market_buy(base_asset, quote_asset, qty).await?;

        // 更新
        self.update_stats_with_order(&order)?;

        // 更新数据库
        let _workflow_id = self.get_context()?.workflow_id();
        let _cloned_db = self.get_context()?.cloned_db();
        let _stats = self.get_stats().lock().await;

        // save to db
        // save_stats(&cloned_db, workflow_id, &stats)?;

        Ok(order)
    }

    async fn market_sell(
        &self,
        client: &SpotClientKind,
        base_asset: &str,
        quote_asset: &str,
        qty: f64,
    ) -> Result<Order> {
        // 提交交易
        let order = client.market_sell(base_asset, quote_asset, qty).await?;

        // 更新
        self.update_stats_with_order(&order)?;

        Ok(order)
    }
}
