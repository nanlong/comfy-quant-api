use crate::{
    data::{ExchangeInfo, Ticker},
    traits::{NodeDataPort, NodeExecutor},
    DataPorts,
};
use anyhow::Result;
use binance::websockets::{WebSockets, WebsocketEvent};
use std::sync::atomic::{AtomicBool, Ordering};
use tokio::sync::broadcast;

struct Widget {
    base_currency: String,
    quote_currency: String,
}

impl Widget {
    pub fn new(base_currency: String, quote_currency: String) -> Self {
        Widget {
            base_currency,
            quote_currency,
        }
    }
}

pub struct BinanceSpotTicker {
    widget: Widget,
    data_ports: DataPorts,
}

impl BinanceSpotTicker {
    pub fn try_new(
        base_currency: impl Into<String>,
        quote_currency: impl Into<String>,
    ) -> Result<Self> {
        let widget = Widget::new(base_currency.into(), quote_currency.into());

        let mut data_ports = DataPorts::new(0, 2);
        data_ports.add_output(0, broadcast::channel::<ExchangeInfo>(1).0)?;
        data_ports.add_output(1, broadcast::channel::<Ticker>(1024).0)?;

        Ok(BinanceSpotTicker { widget, data_ports })
    }

    async fn output0(&self) -> Result<()> {
        let tx = self.data_ports.get_output::<ExchangeInfo>(0)?;

        if tx.receiver_count() > 0 {
            let exchange = ExchangeInfo::new(
                "binance",
                "spot",
                &self.widget.base_currency,
                &self.widget.quote_currency,
            );

            tx.send(exchange)?;
        }

        Ok(())
    }

    async fn output1(&self) -> Result<()> {
        let tx = self.data_ports.get_output::<Ticker>(1)?.clone();
        let keep_running = AtomicBool::new(true);
        let symbol = format!(
            "{}{}@ticker",
            self.widget.base_currency.to_lowercase(),
            self.widget.quote_currency.to_lowercase()
        );

        tokio::spawn(async move {
            let mut web_socket = WebSockets::new(|event: WebsocketEvent| {
                println!("event: {:?}", event);

                if let WebsocketEvent::DayTicker(ticker_event) = event {
                    if let Ok(price) = ticker_event.current_close.parse::<f64>() {
                        let ticker = Ticker {
                            timestamp: ticker_event.event_time,
                            price,
                        };

                        if tx.receiver_count() > 0 {
                            tx.send(ticker).map_err(|e| {
                                binance::errors::Error::from(binance::errors::ErrorKind::Msg(
                                    e.to_string(),
                                ))
                            })?;
                        }
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
        });

        Ok(())
    }
}

impl NodeDataPort for BinanceSpotTicker {
    fn get_data_port(&self) -> Result<&DataPorts> {
        Ok(&self.data_ports)
    }

    fn get_data_port_mut(&mut self) -> Result<&mut DataPorts> {
        Ok(&mut self.data_ports)
    }
}

impl NodeExecutor for BinanceSpotTicker {
    async fn execute(&mut self) -> Result<()> {
        self.output0().await?;
        self.output1().await?;
        Ok(())
    }
}
