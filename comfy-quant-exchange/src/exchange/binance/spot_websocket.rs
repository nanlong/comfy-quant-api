use std::sync::atomic::AtomicBool;

use super::BinanceClient;
use anyhow::Result;
use async_stream::stream;
use binance::websockets::{WebSockets, WebsocketEvent};
use futures::Stream;

#[allow(unused)]
#[derive(Clone)]
pub struct SpotWebsocket<'a> {
    client: &'a BinanceClient,
}

impl<'a> SpotWebsocket<'a> {
    pub fn new(client: &'a BinanceClient) -> Self {
        SpotWebsocket { client }
    }

    pub async fn subscribe(
        &self,
        subscription: impl Into<String>,
    ) -> Result<impl Stream<Item = WebsocketEvent>> {
        let (tx, rx) = flume::unbounded();
        let subscription = subscription.into();
        let config = self.client.config().clone();

        tokio::spawn(async move {
            let keep_running = AtomicBool::new(true);

            let mut websocket = WebSockets::new(|event| {
                let _ = tx.send(event);
                Ok(())
            });

            if let Some(config) = config {
                websocket
                    .connect_with_config(&subscription, &config)
                    .map_err(|_| anyhow::anyhow!("Failed to connect to websocket"))?;
            } else {
                websocket
                    .connect(&subscription)
                    .map_err(|_| anyhow::anyhow!("Failed to connect to websocket"))?;
            }

            if let Err(e) = websocket.event_loop(&keep_running) {
                println!("websocket event loop error: {}", e);
            }

            Ok::<_, anyhow::Error>(())
        });

        let stream = stream! {
            while let Ok(event) = rx.recv_async().await {
                yield event;
            }
        };

        Ok(Box::pin(stream))
    }
}
