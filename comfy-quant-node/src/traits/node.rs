use std::future::Future;

use anyhow::Result;

pub trait Node {
    fn connection<T: Send + Sync + 'static>(
        &self,
        target: &Self,
        origin_slot: usize,
        target_slot: usize,
    ) -> impl Future<Output = Result<()>> + Send;
    fn execute(&self) -> impl Future<Output = Result<()>> + Send;
}
