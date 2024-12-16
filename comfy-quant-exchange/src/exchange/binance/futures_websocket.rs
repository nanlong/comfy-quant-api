use super::BinanceClient;
use anyhow::Result;
use async_stream::stream;
use binance::futures::websockets::{FuturesMarket, FuturesWebSockets, FuturesWebsocketEvent};
use futures::stream::BoxStream;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

#[derive(Debug, Clone, Copy)]
pub enum Market {
    Usdm,
    Coinm,
    Vanilla,
}

impl From<Market> for FuturesMarket {
    fn from(market: Market) -> Self {
        match market {
            Market::Usdm => FuturesMarket::USDM,
            Market::Coinm => FuturesMarket::COINM,
            Market::Vanilla => FuturesMarket::Vanilla,
        }
    }
}

#[allow(unused)]
pub struct FuturesWebsocket<'a> {
    client: &'a BinanceClient,
    market: Market,
    topic: String,
    keep_running: Arc<AtomicBool>,
}

impl<'a> FuturesWebsocket<'a> {
    pub fn new(client: &'a BinanceClient, market: Market, topic: impl Into<String>) -> Self {
        let topic = topic.into();
        let keep_running = Arc::new(AtomicBool::new(true));

        FuturesWebsocket {
            client,
            market,
            topic,
            keep_running,
        }
    }

    pub async fn subscribe(&self) -> Result<BoxStream<FuturesWebsocketEvent>> {
        let (tx, rx) = flume::unbounded();
        let market = self.market.into();
        let topic = self.topic.clone();
        let config = self.client.config().clone();
        let keep_running = self.keep_running.clone();

        tokio::spawn(async move {
            let callback = |event| {
                let _ = tx.send(event);
                Ok(())
            };

            while keep_running.load(Ordering::Relaxed) {
                let mut websocket = FuturesWebSockets::new(callback);

                let resp = if let Some(config) = &config {
                    websocket.connect_with_config(&market, &topic, config)
                } else {
                    websocket.connect(&market, &topic)
                };

                if let Err(e) = resp {
                    tracing::error!("{}", e);
                    std::thread::sleep(std::time::Duration::from_secs(3));
                    continue;
                }

                if let Err(e) = websocket.event_loop(&keep_running) {
                    tracing::error!("{}", e);
                }

                let _ = websocket.disconnect();
            }

            Ok::<(), anyhow::Error>(())
        });

        let stream = stream! {
            while let Ok(event) = rx.recv_async().await {
                yield event;
            }
        };

        Ok(Box::pin(stream))
    }
}

impl Drop for FuturesWebsocket<'_> {
    fn drop(&mut self) {
        self.keep_running.store(false, Ordering::Relaxed);
    }
}
