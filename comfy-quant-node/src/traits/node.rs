use crate::DataPorts;
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

pub trait NodeDataPort {
    fn get_data_port(&self) -> Result<&DataPorts>;

    fn get_data_port_mut(&mut self) -> Result<&mut DataPorts>;
}

pub trait NodeConnector<N: NodeDataPort> {
    fn connection<U: Send + Sync + 'static>(
        &self,
        target: &mut N,
        origin_slot: usize,
        target_slot: usize,
    ) -> impl Future<Output = Result<()>> + Send;
}

impl<T: NodeDataPort + Send + Sync + 'static, N: NodeDataPort + Send + Sync + 'static>
    NodeConnector<N> for T
{
    async fn connection<U: Send + Sync + 'static>(
        &self,
        target: &mut N,
        origin_slot: usize,
        target_slot: usize,
    ) -> Result<()> {
        let origin = self.get_data_port()?;
        let tx = origin.get_output::<U>(origin_slot)?;
        let rx = tx.subscribe();
        target.get_data_port_mut()?.add_input(target_slot, rx)?;

        Ok(())
    }
}
