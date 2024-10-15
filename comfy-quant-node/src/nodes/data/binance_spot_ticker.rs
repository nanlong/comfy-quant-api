use crate::{
    node_core::{Executable, Port, PortAccessor, Slot},
    node_io::SpotPairInfo,
    workflow,
};
use anyhow::Result;
use bon::Builder;

#[derive(Builder, Debug, Clone)]
#[builder(on(String, into))]
pub(crate) struct Params {
    base_asset: String,
    quote_asset: String,
}

/// 币安现货行情
/// outputs:
///      0: SpotPairInfo
///      1: TickStream
#[allow(unused)]
pub(crate) struct BinanceSpotTicker {
    pub(crate) params: Params,
    pub(crate) port: Port,
}

impl BinanceSpotTicker {
    pub(crate) fn try_new(params: Params) -> Result<Self> {
        let mut port = Port::new();

        let pair_info = SpotPairInfo::builder()
            .base_asset(&params.base_asset)
            .quote_asset(&params.quote_asset)
            .build();

        let pair_info_slot = Slot::<SpotPairInfo>::new(pair_info);
        // let output_slot1 = Slot::<Tick>::builder().channel_capacity(1024).build();

        port.add_output(0, pair_info_slot)?;
        // port.add_output(1, output_slot1)?;

        Ok(BinanceSpotTicker { params, port })
    }

    async fn output1(&self) -> Result<()> {
        // let slot = self.port.get_output::<Tick>(1)?;

        // let symbol = format!(
        //     "{}{}@ticker",
        //     self.params.base_asset.to_lowercase(),
        //     self.params.quote_asset.to_lowercase()
        // );

        // todo: 从数据库推送中获取行情
        // tokio::spawn(async move {
        //     loop {
        //         let tick = Tick::builder()
        //             .timestamp(Utc::now().timestamp())
        //             .price(0.)
        //             .build();

        //         slot.send(tick).await?;

        //         tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        //     }

        //     #[allow(unreachable_code)]
        //     Ok::<(), anyhow::Error>(())
        // });

        Ok(())
    }
}

impl PortAccessor for BinanceSpotTicker {
    fn get_port(&self) -> Result<&Port> {
        Ok(&self.port)
    }

    fn get_port_mut(&mut self) -> Result<&mut Port> {
        Ok(&mut self.port)
    }
}

impl Executable for BinanceSpotTicker {
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

        let [base_asset, quote_asset] = node.properties.params.as_slice() else {
            anyhow::bail!("Try from workflow::Node to binanceSpotTicker failed: Invalid params");
        };

        let base_asset = base_asset.as_str().ok_or(anyhow::anyhow!(
            "Try from workflow::Node to binanceSpotTicker failed: Invalid base_asset"
        ))?;

        let quote_asset = quote_asset.as_str().ok_or(anyhow::anyhow!(
            "Try from workflow::Node to binanceSpotTicker failed: Invalid quote_asset"
        ))?;

        let params = Params::builder()
            .base_asset(base_asset)
            .quote_asset(quote_asset)
            .build();

        BinanceSpotTicker::try_new(params)
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

        assert_eq!(binance_spot_ticker.params.base_asset, "BTC");
        assert_eq!(binance_spot_ticker.params.quote_asset, "USDT");
        Ok(())
    }
}
