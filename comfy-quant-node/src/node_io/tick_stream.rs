use crate::node_core::{SymbolPriceStorable, Tick};
use anyhow::Result;
use async_lock::RwLock;
use bon::Builder;
use comfy_quant_exchange::client::spot_client::base::SymbolPrice;
use flume::{Receiver, Sender};
use std::sync::Arc;
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
    pub(crate) async fn save_price(
        &self,
        store: Arc<RwLock<dyn SymbolPriceStorable>>,
    ) -> Result<()> {
        let rx = self.subscribe();
        let cloned_token = self.token.clone();

        tokio::spawn(async move {
            tokio::select! {
                tick = rx.recv_async() => {
                    if let Ok(tick) = tick {
                        let price = SymbolPrice::builder()
                            .symbol(tick.symbol)
                            .price(tick.price)
                            .build();

                        store.write().await.save_price(price)?;
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
    use rust_decimal_macros::dec;
    use std::time::Duration;
    use tokio::time::sleep;

    #[tokio::test]
    async fn test_tick_stream() -> Result<()> {
        let tick_stream = TickStream::new();
        let tick = Tick {
            timestamp: 1,
            symbol: "BTCUSDT".to_string(),
            price: dec!(100.0),
        };

        tick_stream.send(tick.clone()).await?;

        let rx = tick_stream.subscribe();

        let tick2 = rx.recv_async().await?;
        assert_eq!(tick, tick2);

        Ok(())
    }

    #[derive(Debug, Clone, Builder, PartialEq)]
    struct MockSymbolPriceStore {
        prices: Vec<SymbolPrice>,
    }

    impl MockSymbolPriceStore {
        fn new() -> Self {
            MockSymbolPriceStore { prices: vec![] }
        }
    }

    impl SymbolPriceStorable for MockSymbolPriceStore {
        fn save_price(&mut self, price: SymbolPrice) -> Result<()> {
            self.prices.push(price);
            Ok(())
        }
    }

    #[tokio::test]
    async fn test_save_price_should_work() -> Result<()> {
        let tick_stream = TickStream::new();
        let store = Arc::new(RwLock::new(MockSymbolPriceStore::new()));
        let cloned_store = Arc::clone(&store);
        let tick = Tick {
            timestamp: 1,
            symbol: "BTCUSDT".to_string(),
            price: dec!(100.0),
        };

        // 准备保存
        tick_stream.save_price(cloned_store).await?;

        // 发送数据
        tick_stream.send(tick.clone()).await?;

        // 等待保存
        sleep(Duration::from_millis(1)).await;

        let store = store.read().await;
        assert_eq!(store.prices.len(), 1);
        assert_eq!(
            store.prices[0],
            SymbolPrice::builder()
                .symbol("BTCUSDT".to_string())
                .price(dec!(100.0))
                .build()
        );

        Ok(())
    }
}
