use super::Ports;
use anyhow::Result;
use std::future::Future;

pub trait Node {
    fn connection<T: Send + Sync + 'static>(
        &self,
        target: &mut Self,
        origin_slot: usize,
        target_slot: usize,
    ) -> impl Future<Output = Result<()>> + Send;
    fn execute(&self) -> impl Future<Output = Result<()>> + Send;
}

pub trait NodeExecutor {
    fn execute(&mut self) -> impl Future<Output = Result<()>> + Send;
}

pub trait NodePorts {
    fn get_ports(&self) -> Result<&Ports>;

    fn get_ports_mut(&mut self) -> Result<&mut Ports>;
}

pub trait NodeConnector<N: NodePorts> {
    fn connection<U: Clone + Send + Sync + 'static>(
        &self,
        target: &mut N,
        origin_slot: usize,
        target_slot: usize,
    ) -> impl Future<Output = Result<()>> + Send;
}

impl<T: NodePorts + Send + Sync + 'static, N: NodePorts + Send + Sync + 'static> NodeConnector<N>
    for T
{
    async fn connection<U: Clone + Send + Sync + 'static>(
        &self,
        target: &mut N,
        origin_slot: usize,
        target_slot: usize,
    ) -> Result<()> {
        let origin = self.get_ports()?;
        let slot = origin.get_output::<U>(origin_slot)?;
        target.get_ports_mut()?.add_input(target_slot, slot)?;

        Ok(())
    }
}
