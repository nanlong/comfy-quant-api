use super::client::BacktestSpotClient;
use crate::{
    node_core::{NodeCore, NodeExecutable, NodeInfra, TradeStats},
    nodes::{data::BacktestSpotTicker, strategy::SpotGrid},
    workflow::Node,
};
use anyhow::Result;
use enum_dispatch::enum_dispatch;
use rust_decimal::Decimal;
use std::fmt;

#[enum_dispatch(NodeCore, NodeExecutable)]
pub(crate) enum NodeKind {
    // data
    BacktestSpotTicker(BacktestSpotTicker),

    // client
    BacktestSpotClient(BacktestSpotClient),

    // strategy
    SpotGrid(SpotGrid),
}

impl NodeKind {
    fn struct_name(&self) -> &str {
        match self {
            NodeKind::BacktestSpotTicker(_) => "BacktestSpotTicker",
            NodeKind::BacktestSpotClient(_) => "BacktestSpotClient",
            NodeKind::SpotGrid(_) => "SpotGrid",
        }
    }
}

impl TradeStats for NodeKind {
    async fn initial_capital(&self) -> Result<Decimal> {
        match self {
            NodeKind::SpotGrid(spot_grid) => spot_grid.initial_capital().await,
            _ => Ok(Decimal::ZERO),
        }
    }

    async fn realized_pnl(&self) -> Result<Decimal> {
        match self {
            NodeKind::SpotGrid(spot_grid) => spot_grid.realized_pnl().await,
            _ => Ok(Decimal::ZERO),
        }
    }

    async fn unrealized_pnl(&self) -> Result<Decimal> {
        match self {
            NodeKind::SpotGrid(spot_grid) => spot_grid.unrealized_pnl().await,
            _ => Ok(Decimal::ZERO),
        }
    }

    async fn running_time(&self) -> Result<u128> {
        match self {
            NodeKind::SpotGrid(spot_grid) => spot_grid.running_time().await,
            _ => Ok(0),
        }
    }
}

impl fmt::Debug for NodeKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct(self.struct_name()).finish()
    }
}

impl TryFrom<Node> for NodeKind {
    type Error = anyhow::Error;

    fn try_from(node: Node) -> Result<Self> {
        let node_kind = match node.properties.prop_type.as_str() {
            "data.BacktestSpotTicker" => BacktestSpotTicker::try_from(node)?.into(),
            "client.BacktestSpotClient" => BacktestSpotClient::try_from(node)?.into(),
            "strategy.SpotGrid" => SpotGrid::try_from(node)?.into(),
            prop_type => anyhow::bail!("Invalid node type: {}", prop_type),
        };

        Ok(node_kind)
    }
}

impl TryFrom<&NodeKind> for Node {
    type Error = anyhow::Error;

    fn try_from(node_kind: &NodeKind) -> Result<Self> {
        match node_kind {
            NodeKind::BacktestSpotTicker(node) => node.try_into(),
            NodeKind::BacktestSpotClient(node) => node.try_into(),
            NodeKind::SpotGrid(node) => node.try_into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_try_from_workflow_node_to_node_kind() -> Result<()> {
        let json_str = r#"{"id":2,"type":"加密货币交易所/币安现货(Ticker Mock)","pos":[210,58],"size":[240,150],"flags":{},"order":0,"mode":0,"outputs":[{"name":"现货交易对","type":"SpotPairInfo","links":[1],"slot_index":0},{"name":"Tick数据流","type":"TickStream","links":[2],"slot_index":1}],"properties":{"type":"data.BacktestSpotTicker","params":["BTC","USDT","2024-01-01 00:00:00","2024-01-02 00:00:00"]}}"#;
        let node: Node = serde_json::from_str(json_str)?;
        let node_kind = NodeKind::try_from(node)?;

        match node_kind {
            NodeKind::BacktestSpotTicker(_) => {}
            _ => assert!(false),
        }

        Ok(())
    }
}
