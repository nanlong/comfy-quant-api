use anyhow::Result;
use bon::bon;
use flume::{Receiver, Sender};
use std::fmt;

#[derive(Clone, Debug)]
pub struct Slot<T> {
    data: Option<T>,
    channel: Option<(Sender<T>, Receiver<T>)>,
}

#[bon]
#[allow(unused)]
impl<T> Slot<T>
where
    T: Clone + fmt::Debug + Send + Sync + 'static,
{
    #[builder]
    pub fn new(data: Option<T>, channel_capacity: Option<usize>) -> Self {
        let channel = channel_capacity.and_then(|capacity| {
            let (tx, rx) = flume::bounded::<T>(capacity);
            Some((tx, rx))
        });

        Self { data, channel }
    }

    // 访问数据
    pub fn data(&self) -> Option<&T> {
        self.data.as_ref()
    }

    // 发送数据
    pub async fn send(&self, data: T) -> Result<()> {
        self.channel
            .as_ref()
            .ok_or(anyhow::anyhow!("No channel to send to"))?
            .0
            .send_async(data)
            .await
            .map_err(|e| anyhow::anyhow!(e))
    }

    // 订阅数据
    pub fn subscribe(&self) -> Result<Receiver<T>> {
        let rx = self
            .channel
            .as_ref()
            .ok_or(anyhow::anyhow!("No channel to subscribe to"))?
            .1
            .clone();

        Ok(rx)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_slot_builder() -> Result<()> {
        let slot = Slot::<usize>::builder().channel_capacity(10).build();
        assert_eq!(slot.data(), None);
        let rx = slot.subscribe()?;
        slot.send(10).await?;
        let data = rx.recv_async().await?;
        assert_eq!(data, 10);

        let slot = Slot::<usize>::builder().data(10).build();
        assert_eq!(slot.data(), Some(&10));
        assert!(slot.subscribe().is_err());

        let slot = Slot::<usize>::builder()
            .data(10)
            .channel_capacity(10)
            .build();
        assert_eq!(slot.data(), Some(&10));
        let rx = slot.subscribe()?;
        slot.send(8).await?;
        let data = rx.recv_async().await?;
        assert_eq!(data, 8);

        Ok(())
    }
}
