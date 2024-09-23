use crate::{
    data::{CryptoExchange, Ticker},
    data_ports::DataPorts,
    traits::Node,
};
use anyhow::Result;
use binance::websockets::{WebSockets, WebsocketEvent};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use tokio::sync::{broadcast, Mutex};

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
    data_ports: Arc<Mutex<DataPorts<0, 2>>>,
}

impl BinanceSpotTicker {
    pub fn try_new(base_currency: String, quote_currency: String) -> Result<Self> {
        let widget = Widget::new(base_currency, quote_currency);

        let mut data_ports = DataPorts::<0, 2>::new();
        data_ports.add_output(0, broadcast::channel::<CryptoExchange>(1).0)?;
        data_ports.add_output(1, broadcast::channel::<Ticker>(1).0)?;

        Ok(BinanceSpotTicker {
            widget,
            data_ports: Arc::new(Mutex::new(data_ports)),
        })
    }

    async fn output0(&self) -> Result<()> {
        let data_ports = self.data_ports.lock().await;
        let tx = data_ports.get_output::<CryptoExchange>(0)?;
        let exchange = CryptoExchange::new(
            "binance",
            "spot",
            &self.widget.base_currency,
            &self.widget.quote_currency,
        );

        tx.send(exchange)?;
        Ok(())
    }

    async fn output1(&self) -> Result<()> {
        let data_ports = self.data_ports.lock().await;
        let tx = data_ports.get_output::<Ticker>(1)?.clone();
        let keep_running = AtomicBool::new(true);
        let symbol = format!(
            "{}{}@ticker",
            self.widget.base_currency.to_lowercase(),
            self.widget.quote_currency.to_lowercase()
        );

        tokio::spawn(async move {
            let mut web_socket = WebSockets::new(|event: WebsocketEvent| {
                if let WebsocketEvent::DayTicker(ticker_event) = event {
                    if let Ok(price) = ticker_event.current_close.parse::<f64>() {
                        let ticker = Ticker {
                            timestamp: ticker_event.event_time,
                            price,
                        };

                        tx.send(ticker).map_err(|e| {
                            binance::errors::Error::from(binance::errors::ErrorKind::Msg(
                                e.to_string(),
                            ))
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
        });

        Ok(())
    }
}

impl Node for BinanceSpotTicker {
    async fn connection<T: Send + Sync + 'static>(
        &self,
        target: &Self,
        origin_slot: usize,
        target_slot: usize,
    ) -> Result<()> {
        let origin = self.data_ports.lock().await;
        let mut target = target.data_ports.lock().await;

        let tx = origin.get_output::<T>(origin_slot)?;
        let rx = tx.subscribe();
        target.add_input(target_slot, rx)?;

        Ok(())
    }

    async fn execute(&self) -> Result<()> {
        self.output0().await?;
        self.output1().await?;
        Ok(())
    }
}
