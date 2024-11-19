use std::sync::Arc;

use crate::node_core::{Tick, TickStore};
use anyhow::Result;
use async_lock::Mutex;
use bon::Builder;
use flume::{Receiver, Sender};
use tokio_util::sync::CancellationToken;

#[derive(Debug, Builder)]
pub(crate) struct TickStream {
    inner: (Sender<Tick>, Receiver<Tick>),
    token: CancellationToken,
}

impl TickStream {
    pub(crate) fn new() -> Self {
        TickStream {
            inner: flume::unbounded(),
            token: CancellationToken::new(),
        }
    }

    pub(crate) async fn send(&self, tick: Tick) -> Result<()> {
        self.inner.0.send_async(tick).await?;
        Ok(())
    }

    pub(crate) fn subscribe(&self) -> Receiver<Tick> {
        self.inner.1.clone()
    }

    #[allow(unused)]
    pub(crate) async fn save_price(&self, store: Arc<Mutex<dyn TickStore>>) -> Result<()> {
        let rx = self.subscribe();
        let cloned_token = self.token.clone();

        tokio::spawn(async move {
            tokio::select! {
                tick = rx.recv_async() => {
                    if let Ok(tick) = tick {
                        store.lock().await.save_price(tick)?;
                    }

                    Ok::<_, anyhow::Error>(())
                }

                _ = cloned_token.cancelled() => {
                    Ok(())
                }
            }
        });

        Ok(())
    }
}

impl Drop for TickStream {
    fn drop(&mut self) {
        self.token.cancel();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal::Decimal;
    use std::time::Duration;
    use tokio::time::sleep;

    #[tokio::test]
    async fn test_tick_stream() -> Result<()> {
        let tick_stream = TickStream::new();
        let tick = Tick {
            timestamp: 1,
            price: Decimal::try_from(100.0).unwrap(),
        };

        tick_stream.send(tick.clone()).await?;

        let rx = tick_stream.subscribe();

        let tick2 = rx.recv_async().await?;
        assert_eq!(tick, tick2);

        Ok(())
    }

    #[derive(Debug, Clone, Builder, PartialEq)]
    struct MockTickStore {
        ticks: Vec<Tick>,
    }

    impl MockTickStore {
        fn new() -> Self {
            MockTickStore { ticks: vec![] }
        }
    }

    impl TickStore for MockTickStore {
        fn save_price(&mut self, tick: Tick) -> Result<()> {
            self.ticks.push(tick);
            Ok(())
        }
    }

    #[tokio::test]
    async fn test_save_price() -> Result<()> {
        let tick_stream = TickStream::new();
        let store = Arc::new(Mutex::new(MockTickStore::new()));
        let cloned_store = Arc::clone(&store);
        let tick = Tick {
            timestamp: 1,
            price: Decimal::try_from(100.0).unwrap(),
        };

        // 准备保存
        tick_stream.save_price(cloned_store).await?;

        // 发送数据
        tick_stream.send(tick.clone()).await?;

        // 等待保存
        sleep(Duration::from_millis(1)).await;

        let store = store.lock().await;
        assert_eq!(store.ticks.len(), 1);
        assert_eq!(store.ticks[0], tick);

        Ok(())
    }
}
