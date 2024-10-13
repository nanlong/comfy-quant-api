use crate::base::Ports;
use anyhow::Result;
use enum_dispatch::enum_dispatch;

// 节点执行
#[enum_dispatch]
#[allow(async_fn_in_trait)]
pub trait NodeExecutor {
    async fn execute(&mut self) -> Result<()>;
}

// 节点插槽
#[enum_dispatch]
pub trait NodePorts {
    fn get_ports(&self) -> Result<&Ports>;

    fn get_ports_mut(&mut self) -> Result<&mut Ports>;
}

// 节点连接
#[enum_dispatch]
#[allow(async_fn_in_trait)]
pub trait NodeConnector {
    fn connection<U>(
        &self,                      // 当前节点
        target: &mut dyn NodePorts, // 目标节点
        origin_slot: usize,         // 当前节点输出槽位
        target_slot: usize,         // 目标节点输入槽位
    ) -> Result<()>
    where
        U: Clone + Send + Sync + 'static;
}

// 节点连接默认实现
impl<T> NodeConnector for T
where
    T: NodePorts + Send + Sync + 'static,
{
    fn connection<U>(
        &self,                      // 当前节点
        target: &mut dyn NodePorts, // 目标节点
        origin_slot: usize,         // 当前节点输出槽位
        target_slot: usize,         // 目标节点输入槽位
    ) -> Result<()>
    where
        U: Clone + Send + Sync + 'static,
    {
        let origin = self.get_ports()?;
        let slot = origin.get_output::<U>(origin_slot)?;
        target.get_ports_mut()?.add_input(target_slot, slot)?;

        Ok(())
    }
}
