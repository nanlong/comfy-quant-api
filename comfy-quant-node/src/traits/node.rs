use anyhow::Result;
use std::future::Future;
use tokio::sync::mpsc;

pub trait Node<Input> {
    type Output;

    fn execute(
        &self,
        input: Input,
        tx: mpsc::UnboundedSender<Self::Output>,
    ) -> impl Future<Output = Result<()>> + Send;
}
