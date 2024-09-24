use crate::{
    data::{ExchangeInfo, Ticker},
    traits::{NodeDataPort, NodeExecutor},
    workflow, DataPorts,
};
use anyhow::Result;
use binance::websockets::{WebSockets, WebsocketEvent};
use std::sync::atomic::{AtomicBool, Ordering};
use tokio::sync::broadcast;

pub struct Widget {
    base_currency: String,
    quote_currency: String,
}

impl Widget {
    pub fn new(base_currency: impl Into<String>, quote_currency: impl Into<String>) -> Self {
        Widget {
            base_currency: base_currency.into(),
            quote_currency: quote_currency.into(),
        }
    }
}

pub struct BinanceSpotTicker {
    pub(crate) widget: Widget,
    pub(crate) data_ports: DataPorts,
}

impl BinanceSpotTicker {
    pub fn try_new(widget: Widget) -> Result<Self> {
        let mut data_ports = DataPorts::new(0, 2);
        data_ports.add_output(0, broadcast::channel::<ExchangeInfo>(1).0)?;
        data_ports.add_output(1, broadcast::channel::<Ticker>(1024).0)?;

        Ok(BinanceSpotTicker { widget, data_ports })
    }

    async fn output0(&self) -> Result<()> {
        let tx = self.data_ports.get_output::<ExchangeInfo>(0)?.clone();

        let exchange = ExchangeInfo::new(
            "binance",
            "spot",
            &self.widget.base_currency,
            &self.widget.quote_currency,
        );

        tokio::spawn(async move {
            while tx.receiver_count() > 0 {
                tx.send(exchange)?;
                break;
            }

            Ok::<(), anyhow::Error>(())
        });

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

impl TryFrom<workflow::Node> for BinanceSpotTicker {
    type Error = anyhow::Error;

    fn try_from(node: workflow::Node) -> Result<Self> {
        if node.properties.prop_type != "data.binanceSpotTicker" {
            anyhow::bail!("Try from workflow::Node to binanceSpotTicker failed: Invalid prop_type");
        }

        let [base_currency, quote_currency] = node.properties.params.as_slice() else {
            anyhow::bail!("Try from workflow::Node to binanceSpotTicker failed: Invalid params");
        };

        let base_currency = base_currency.as_str().ok_or(anyhow::anyhow!(
            "Try from workflow::Node to binanceSpotTicker failed: Invalid base_currency"
        ))?;

        let quote_currency = quote_currency.as_str().ok_or(anyhow::anyhow!(
            "Try from workflow::Node to binanceSpotTicker failed: Invalid quote_currency"
        ))?;

        let widget = Widget::new(base_currency, quote_currency);
        BinanceSpotTicker::try_new(widget)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_try_from_node_to_binance_spot_ticker() -> anyhow::Result<()> {
        let json_str = r#"{"id":1,"type":"数据/币安现货行情","pos":[199,74],"size":{"0":210,"1":310},"flags":{},"order":0,"mode":0,"inputs":[],"properties":{"type":"data.binanceSpotTicker","params":["BTC","USDT"]}}"#;

        let node: workflow::Node = serde_json::from_str(json_str)?;
        let binance_spot_ticker = BinanceSpotTicker::try_from(node)?;

        assert_eq!(binance_spot_ticker.widget.base_currency, "BTC");
        assert_eq!(binance_spot_ticker.widget.quote_currency, "USDT");
        Ok(())
    }
}
