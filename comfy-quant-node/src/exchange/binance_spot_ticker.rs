use crate::{
    base::{
        traits::node::{NodeExecutor, NodePorts},
        Ports, Slot,
    },
    data::{ExchangeInfo, Ticker},
    workflow,
};
use anyhow::Result;
use bon::Builder;
use chrono::Utc;

#[derive(Builder, Debug, Clone)]
#[builder(on(String, into))]
pub struct Widget {
    base_currency: String,
    quote_currency: String,
}

#[allow(unused)]
pub struct BinanceSpotTicker {
    pub(crate) widget: Widget,
    pub(crate) ports: Ports,
}

impl BinanceSpotTicker {
    pub fn try_new(widget: Widget) -> Result<Self> {
        let mut ports = Ports::new();

        let exchange_info = ExchangeInfo::builder()
            .name("binance")
            .market("spot")
            .base_currency(&widget.base_currency)
            .quote_currency(&widget.quote_currency)
            .build();

        ports.add_output(
            0,
            Slot::<ExchangeInfo>::builder().data(exchange_info).build(),
        )?;

        ports.add_output(1, Slot::<Ticker>::builder().channel_capacity(1024).build())?;

        Ok(BinanceSpotTicker { widget, ports })
    }

    async fn output1(&self) -> Result<()> {
        let slot = self.ports.get_output::<Ticker>(1)?;

        // let symbol = format!(
        //     "{}{}@ticker",
        //     self.widget.base_currency.to_lowercase(),
        //     self.widget.quote_currency.to_lowercase()
        // );

        // todo: 从数据库推送中获取行情
        tokio::spawn(async move {
            loop {
                let ticker = Ticker::builder()
                    .timestamp(Utc::now().timestamp())
                    .price(0.)
                    .build();

                slot.send(ticker)?;

                tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
            }

            #[allow(unreachable_code)]
            Ok::<(), anyhow::Error>(())
        });

        Ok(())
    }
}

impl NodePorts for BinanceSpotTicker {
    fn get_ports(&self) -> Result<&Ports> {
        Ok(&self.ports)
    }

    fn get_ports_mut(&mut self) -> Result<&mut Ports> {
        Ok(&mut self.ports)
    }
}

impl NodeExecutor for BinanceSpotTicker {
    async fn execute(&mut self) -> Result<()> {
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

        let widget = Widget::builder()
            .base_currency(base_currency)
            .quote_currency(quote_currency)
            .build();

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
