use crate::base::Ports;
use anyhow::Result;
use std::future::Future;

// 节点执行
pub trait NodeExecutor {
    fn execute(&mut self) -> impl Future<Output = Result<()>> + Send;
}

// 节点插槽
pub trait NodePorts {
    fn get_ports(&self) -> Result<&Ports>;

    fn get_ports_mut(&mut self) -> Result<&mut Ports>;
}

// 节点连接
pub trait NodeConnector<N>
where
    N: NodePorts,
{
    fn connection<U>(
        &self,              // 当前节点
        target: &mut N,     // 目标节点
        origin_slot: usize, // 当前节点输出槽位
        target_slot: usize, // 目标节点输入槽位
    ) -> impl Future<Output = Result<()>> + Send
    where
        U: Clone + Send + Sync + 'static;
}

// 节点连接默认实现
impl<T, N> NodeConnector<N> for T
where
    T: NodePorts + Send + Sync + 'static,
    N: NodePorts + Send + Sync + 'static,
{
    async fn connection<U>(
        &self,              // 当前节点
        target: &mut N,     // 目标节点
        origin_slot: usize, // 当前节点输出槽位
        target_slot: usize, // 目标节点输入槽位
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
