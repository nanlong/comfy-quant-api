use crate::{node_core::Port, workflow::WorkflowContext};
use anyhow::Result;
use comfy_quant_exchange::client::{
    spot_client::base::Order,
    spot_client_kind::{SpotClientExecutable, SpotClientKind},
};
use enum_dispatch::enum_dispatch;

#[enum_dispatch]
pub trait Setupable {
    fn setup_context(&mut self, context: WorkflowContext);

    fn get_context(&self) -> Result<&WorkflowContext>;
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
    fn get_port(&self) -> Result<&Port>;

    fn get_port_mut(&mut self) -> Result<&mut Port>;
}

// 节点连接
#[enum_dispatch]
pub trait Connectable {
    fn connection<U>(
        &self,                         // 当前节点
        target: &mut dyn PortAccessor, // 目标节点
        origin_slot: usize,            // 当前节点输出槽位
        target_slot: usize,            // 目标节点输入槽位
    ) -> Result<()>
    where
        U: Clone + Send + Sync + 'static;
}

// 节点连接默认实现
impl<T> Connectable for T
where
    T: PortAccessor + Send + Sync + 'static,
{
    fn connection<U>(
        &self,                         // 当前节点
        target: &mut dyn PortAccessor, // 目标节点
        origin_slot: usize,            // 当前节点输出槽位
        target_slot: usize,            // 目标节点输入槽位
    ) -> Result<()>
    where
        U: Clone + Send + Sync + 'static,
    {
        let origin = self.get_port()?;
        let slot = origin.get_output::<U>(origin_slot)?;
        target.get_port_mut()?.add_input(target_slot, slot)?;

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

impl<T> SpotTradeable for T
where
    T: Setupable,
{
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
        self.get_context()?.update_stats_with_order(&order)?;

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
        self.get_context()?.update_stats_with_order(&order)?;

        Ok(order)
    }
}
