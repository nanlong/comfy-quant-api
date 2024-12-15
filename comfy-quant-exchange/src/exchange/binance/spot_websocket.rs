use super::BinanceClient;
use anyhow::Result;
use async_stream::stream;
use binance::websockets::{WebSockets, WebsocketEvent};
use futures::stream::BoxStream;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

#[allow(unused)]
pub struct SpotWebsocket<'a> {
    client: &'a BinanceClient,
    topic: String,
    keep_running: Arc<AtomicBool>,
}

impl<'a> SpotWebsocket<'a> {
    pub fn new(client: &'a BinanceClient, topic: impl Into<String>) -> Self {
        let topic = topic.into();
        let keep_running = Arc::new(AtomicBool::new(true));

        SpotWebsocket {
            client,
            topic,
            keep_running,
        }
    }

    pub async fn subscribe(&self) -> Result<BoxStream<WebsocketEvent>> {
        let (tx, rx) = flume::unbounded();
        let topic = self.topic.clone();
        let config = self.client.config().clone();
        let keep_running = self.keep_running.clone();

        tokio::spawn(async move {
            let callback = |event| {
                let _ = tx.send(event);
                Ok(())
            };

            while keep_running.load(Ordering::Relaxed) {
                let mut websocket = WebSockets::new(callback);

                let resp = if let Some(config) = &config {
                    websocket.connect_with_config(&topic, config)
                } else {
                    websocket.connect(&topic)
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

impl Drop for SpotWebsocket<'_> {
    fn drop(&mut self) {
        self.keep_running.store(false, Ordering::Relaxed);
    }
}
