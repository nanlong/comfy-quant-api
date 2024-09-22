use super::ticker::Ticker;
use crate::traits::node::Node;
use anyhow::Result;
use binance::websockets::{WebSockets, WebsocketEvent};
use std::sync::atomic::{AtomicBool, Ordering};
use tokio::sync::mpsc;

pub struct BinanceSpotTicker {}

impl Node<BinanceSpotTickerInput> for BinanceSpotTicker {
    type Output = Ticker;

    async fn execute(
        &self,
        input: BinanceSpotTickerInput,
        tx: mpsc::UnboundedSender<Self::Output>,
    ) -> Result<()> {
        // let (tx, rx) = mpsc::unbounded_channel();
        let keep_running = AtomicBool::new(true);
        let symbol = format!(
            "{}{}@ticker",
            input.base_currency.to_lowercase(),
            input.quote_currency.to_lowercase()
        );

        let mut web_socket = WebSockets::new(|event: WebsocketEvent| {
            println!("{:?}", event);

            if let WebsocketEvent::DayTicker(ticker_event) = event {
                if let Ok(price) = ticker_event.current_close.parse::<f64>() {
                    let ticker = Ticker {
                        timestamp: ticker_event.event_time,
                        price,
                    };

                    tx.send(ticker).map_err(|e| {
                        binance::errors::Error::from(binance::errors::ErrorKind::Msg(e.to_string()))
                    })?;
                }
            }

            Ok(())
        });

        web_socket
            .connect(&symbol)
            .map_err(|e| anyhow::anyhow!(e.to_string()))?;

        while keep_running.load(Ordering::Relaxed) {
            if let Err(e) = web_socket.event_loop(&keep_running) {
                return Err(anyhow::anyhow!(e.to_string()));
            }
        }

        web_socket
            .disconnect()
            .map_err(|e| anyhow::anyhow!(e.to_string()))?;

        Ok(())
    }
}

pub struct BinanceSpotTickerInput {
    pub base_currency: String,
    pub quote_currency: String,
}
