use crate::node_core::{SymbolPriceStorable, Tick};
use anyhow::Result;
use async_lock::RwLock;
use bon::Builder;
use comfy_quant_exchange::client::spot_client::base::Exchange;
use flume::{Receiver, Sender};
use std::sync::Arc;
use tokio_util::sync::CancellationToken;

type ExchangeTick = (Exchange, Tick);

#[derive(Debug, Builder)]
pub(crate) struct TickStream {
    inner: (Sender<ExchangeTick>, Receiver<ExchangeTick>),
    token: CancellationToken,
}

impl TickStream {
    pub(crate) fn new() -> Self {
        TickStream {
            inner: flume::unbounded(),
            token: CancellationToken::new(),
        }
    }

    pub(crate) async fn send(&self, exchange: impl Into<Exchange>, tick: Tick) -> Result<()> {
        self.inner.0.send_async((exchange.into(), tick)).await?;
        Ok(())
    }

    pub(crate) fn subscribe(&self) -> Receiver<ExchangeTick> {
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
                resp = rx.recv_async() => {
                    if let Ok((exchange, tick)) = resp {
                        store.write().await.save_price(exchange, tick.into())?;
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
    use comfy_quant_exchange::client::spot_client::base::SymbolPrice;
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
        let exchange = Exchange::new("Binance");

        tick_stream.send(exchange.clone(), tick.clone()).await?;

        let rx = tick_stream.subscribe();

        let tick2 = rx.recv_async().await?;
        assert_eq!((exchange, tick), tick2);

        Ok(())
    }

    #[derive(Debug, Clone, Builder, PartialEq)]
    struct MockExchangeSymbolPriceStore {
        prices: Vec<(Exchange, SymbolPrice)>,
    }

    impl MockExchangeSymbolPriceStore {
        fn new() -> Self {
            MockExchangeSymbolPriceStore { prices: vec![] }
        }
    }

    impl SymbolPriceStorable for MockExchangeSymbolPriceStore {
        fn save_price(&mut self, exchange: Exchange, price: SymbolPrice) -> Result<()> {
            self.prices.push((exchange, price));
            Ok(())
        }
    }

    #[tokio::test]
    async fn test_save_price_should_work() -> Result<()> {
        let tick_stream = TickStream::new();
        let store = Arc::new(RwLock::new(MockExchangeSymbolPriceStore::new()));
        let cloned_store = Arc::clone(&store);
        let tick = Tick {
            timestamp: 1,
            symbol: "BTCUSDT".to_string(),
            price: dec!(100.0),
        };
        let exchange = Exchange::new("Binance");

        // 准备保存
        tick_stream.save_price(cloned_store).await?;

        // 发送数据
        tick_stream.send(exchange.clone(), tick.clone()).await?;

        // 等待保存
        sleep(Duration::from_millis(1)).await;

        let store = store.read().await;
        assert_eq!(store.prices.len(), 1);
        assert_eq!(store.prices[0], (exchange, tick.into()));

        Ok(())
    }
}
