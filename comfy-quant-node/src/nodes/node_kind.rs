use super::client::BacktestSpotClient;
use crate::{
    node_core::{Executable, Port, PortAccessor, Setupable},
    nodes::{data::BacktestSpotTicker, strategy::SpotGrid},
    workflow::{self, WorkflowContext},
};
use anyhow::Result;
use enum_dispatch::enum_dispatch;
use std::sync::Arc;

#[derive(Debug)]
#[enum_dispatch(Setupable, PortAccessor, Executable)]
pub enum NodeKind {
    // data
    BacktestSpotTicker(BacktestSpotTicker),

    // client
    BacktestSpotClient(BacktestSpotClient),

    // strategy
    SpotGrid(SpotGrid),
}

impl TryFrom<&workflow::Node> for NodeKind {
    type Error = anyhow::Error;

    fn try_from(node: &workflow::Node) -> Result<Self> {
        let node_kind = match node.properties.prop_type.as_str() {
            "data.BacktestSpotTicker" => BacktestSpotTicker::try_from(node)?.into(),
            "client.BacktestSpotClient" => BacktestSpotClient::try_from(node)?.into(),
            "strategy.SpotGrid" => SpotGrid::try_from(node)?.into(),
            prop_type => anyhow::bail!("Invalid node type: {}", prop_type),
        };

        Ok(node_kind)
    }
}

#[cfg(test)]
mod tests {
    use crate::utils;

    use super::*;

    #[test]
    fn test_try_from_workflow_node_to_node_kind() -> Result<()> {
        let json_str = r#"{"id":2,"type":"加密货币交易所/币安现货(Ticker Mock)","pos":[210,58],"size":[240,150],"flags":{},"order":0,"mode":0,"outputs":[{"name":"现货交易对","type":"SpotPairInfo","links":[1],"slot_index":0},{"name":"Tick数据流","type":"TickStream","links":[2],"slot_index":1}],"properties":{"type":"data.BacktestSpotTicker","params":["BTC","USDT","2024-01-01 00:00:00","2024-01-02 00:00:00"]}}"#;
        let node: workflow::Node = serde_json::from_str(json_str)?;
        let node_kind = NodeKind::try_from(&node)?;

        match node_kind {
            NodeKind::BacktestSpotTicker(node) => {
                assert_eq!(node.params.base_asset, "BTC");
                assert_eq!(node.params.quote_asset, "USDT");
                assert_eq!(
                    node.params.start_datetime,
                    utils::add_utc_offset("2024-01-01 00:00:00")?
                );
                assert_eq!(
                    node.params.end_datetime,
                    utils::add_utc_offset("2024-01-02 00:00:00")?
                );
            }
            _ => assert!(false),
        }

        Ok(())
    }
}
