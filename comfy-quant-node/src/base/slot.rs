use anyhow::Result;
use bon::bon;
use std::fmt;
use tokio::sync::broadcast::{self, Receiver, Sender};

#[derive(Clone, Debug)]
pub struct Slot<T> {
    data: Option<T>,
    tx: Option<Sender<T>>,
}

#[bon]
#[allow(unused)]
impl<T> Slot<T>
where
    T: Clone + fmt::Debug + Send + Sync + 'static,
{
    #[builder]
    pub fn new(data: Option<T>, channel_capacity: Option<usize>) -> Self {
        let tx = channel_capacity.and_then(|capacity| {
            let (tx, _) = broadcast::channel::<T>(capacity);
            Some(tx)
        });

        Self { data, tx }
    }

    // 访问数据
    pub fn data(&self) -> Option<&T> {
        self.data.as_ref()
    }

    // 发送数据
    pub fn send(&self, data: T) -> Result<usize> {
        self.tx
            .as_ref()
            .ok_or(anyhow::anyhow!("No channel to send to"))?
            .send(data)
            .map_err(|e| anyhow::anyhow!(e))
    }

    // 订阅数据
    pub fn subscribe(&self) -> Result<Receiver<T>> {
        let rx = self
            .tx
            .as_ref()
            .ok_or(anyhow::anyhow!("No channel to subscribe to"))?
            .subscribe();

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
        let mut tx = slot.subscribe()?;
        slot.send(10)?;
        let data = tx.recv().await?;
        assert_eq!(data, 10);

        let slot = Slot::<usize>::builder().data(10).build();
        assert_eq!(slot.data(), Some(&10));
        assert!(slot.subscribe().is_err());

        let slot = Slot::<usize>::builder()
            .data(10)
            .channel_capacity(10)
            .build();
        assert_eq!(slot.data(), Some(&10));
        let mut tx = slot.subscribe()?;
        slot.send(8)?;
        let data = tx.recv().await?;
        assert_eq!(data, 8);

        Ok(())
    }
}
