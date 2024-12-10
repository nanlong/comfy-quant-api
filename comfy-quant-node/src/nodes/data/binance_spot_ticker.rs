use crate::{
    node_core::{NodeCore, NodeCoreExt, NodeExecutable, NodeInfra, Slot},
    node_io::SpotPairInfo,
    workflow::Node,
};
use anyhow::Result;
use bon::Builder;
use std::sync::Arc;

/// 币安现货行情
/// outputs:
///      0: SpotPairInfo
///      1: TickStream
#[derive(Debug)]
#[allow(unused)]
pub(crate) struct BinanceSpotTicker {
    params: Params,
    infra: NodeInfra,
}

impl NodeCore for BinanceSpotTicker {
    fn node_infra(&self) -> &NodeInfra {
        &self.infra
    }

    fn node_infra_mut(&mut self) -> &mut NodeInfra {
        &mut self.infra
    }
}

impl BinanceSpotTicker {
    pub(crate) fn try_new(node: Node) -> Result<Self> {
        let params = Params::try_from(&node)?;
        let infra = NodeInfra::new(node);

        Ok(BinanceSpotTicker { params, infra })
    }

    async fn output1(&self) -> Result<()> {
        // let slot = self.port.output::<Tick>(1)?;

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

impl NodeExecutable for BinanceSpotTicker {
    async fn initialize(&mut self) -> Result<()> {
        let pair_info = SpotPairInfo::builder()
            .base_asset(&self.params.base_asset)
            .quote_asset(&self.params.quote_asset)
            .build();

        let pair_info_slot = Arc::new(Slot::<SpotPairInfo>::new(pair_info));
        // let output_slot1 = Slot::<Tick>::builder().channel_capacity(1024).build();

        self.port_mut().set_output(0, pair_info_slot)?;
        // port.set_output(1, output_slot1)?;

        Ok(())
    }

    async fn execute(&mut self) -> Result<()> {
        self.output1().await?;
        Ok(())
    }
}

impl TryFrom<Node> for BinanceSpotTicker {
    type Error = anyhow::Error;

    fn try_from(node: Node) -> Result<Self> {
        BinanceSpotTicker::try_new(node)
    }
}

#[derive(Builder, Debug, Clone)]
#[builder(on(String, into))]
pub(crate) struct Params {
    base_asset: String,
    quote_asset: String,
}

impl TryFrom<&Node> for Params {
    type Error = BinanceSpotTickerError;

    fn try_from(node: &Node) -> Result<Self, Self::Error> {
        if node.properties.prop_type != "data.BinanceSpotTicker" {
            return Err(BinanceSpotTickerError::PropertyTypeMismatch);
        }

        let [base_asset, quote_asset] = node.properties.params.as_slice() else {
            return Err(BinanceSpotTickerError::ParamsFormatError);
        };

        let base_asset = base_asset
            .as_str()
            .ok_or(BinanceSpotTickerError::BaseAssetError)?;

        let quote_asset = quote_asset
            .as_str()
            .ok_or(BinanceSpotTickerError::QuoteAssetError)?;

        let params = Params::builder()
            .base_asset(base_asset)
            .quote_asset(quote_asset)
            .build();

        Ok(params)
    }
}

#[derive(thiserror::Error, Debug)]
pub enum BinanceSpotTickerError {
    #[error("Invalid property type, expected 'data.BinanceSpotTicker'")]
    PropertyTypeMismatch,

    #[error("Invalid parameters format")]
    ParamsFormatError,

    #[error("Invalid base asset")]
    BaseAssetError,

    #[error("Invalid quote asset")]
    QuoteAssetError,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_try_from_node_to_binance_spot_ticker() -> anyhow::Result<()> {
        let json_str = r#"{"id":1,"type":"数据/币安现货行情","pos":[199,74],"size":{"0":210,"1":310},"flags":{},"order":0,"mode":0,"inputs":[],"properties":{"type":"data.BinanceSpotTicker","params":["BTC","USDT"]}}"#;

        let node: Node = serde_json::from_str(json_str)?;
        let binance_spot_ticker = BinanceSpotTicker::try_from(node)?;

        assert_eq!(binance_spot_ticker.params.base_asset, "BTC");
        assert_eq!(binance_spot_ticker.params.quote_asset, "USDT");
        Ok(())
    }
}
