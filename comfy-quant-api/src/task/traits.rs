use super::status::TaskStatus;
use anyhow::Result;
use flume::Receiver;

#[allow(async_fn_in_trait)]
pub trait Task {
    async fn run(self) -> Result<Receiver<TaskStatus>>;
}
