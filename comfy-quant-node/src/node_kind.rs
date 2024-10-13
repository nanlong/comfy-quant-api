use crate::{
    account::MockSpotAccount,
    base::{
        traits::{NodeExecutor, NodePorts},
        Ports,
    },
    exchange::BinanceSpotTickerMock,
    strategy::SpotGrid,
    workflow,
};
use anyhow::Result;
use enum_dispatch::enum_dispatch;

#[derive(Debug)]
#[enum_dispatch(NodePorts, NodeExecutor)]
pub enum NodeKind {
    BinanceSpotTickerMock(BinanceSpotTickerMock),
    MockSpotAccount(MockSpotAccount),
    SpotGrid(SpotGrid),
}

impl TryFrom<workflow::Node> for NodeKind {
    type Error = anyhow::Error;

    fn try_from(node: workflow::Node) -> Result<Self> {
        match node.properties.prop_type.as_str() {
            "data.BinanceSpotTickerMock" => Ok(Self::BinanceSpotTickerMock(
                BinanceSpotTickerMock::try_from(node)?,
            )),
            "account.mockSpotAccount" => {
                Ok(Self::MockSpotAccount(MockSpotAccount::try_from(node)?))
            }
            "strategy.spotGrid" => Ok(Self::SpotGrid(SpotGrid::try_from(node)?)),
            _ => anyhow::bail!("Invalid node type"),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::utils;

    use super::*;

    #[test]
    fn test_try_from_workflow_node_to_node_kind() -> Result<()> {
        let json_str = r#"{"id":2,"type":"加密货币交易所/币安现货(Ticker Mock)","pos":[210,58],"size":[240,150],"flags":{},"order":0,"mode":0,"outputs":[{"name":"现货交易对","type":"SpotPairInfo","links":[1],"slot_index":0},{"name":"Tick数据流","type":"TickStream","links":[2],"slot_index":1}],"properties":{"type":"data.BinanceSpotTickerMock","params":["BTC","USDT","2024-01-01 00:00:00","2024-01-02 00:00:00"]}}"#;
        let node: workflow::Node = serde_json::from_str(json_str)?;
        let node_kind = NodeKind::try_from(node)?;

        match node_kind {
            NodeKind::BinanceSpotTickerMock(node) => {
                assert_eq!(node.widget.base_currency, "BTC");
                assert_eq!(node.widget.quote_currency, "USDT");
                assert_eq!(
                    node.widget.start_datetime,
                    utils::add_utc_offset("2024-01-01 00:00:00")?
                );
                assert_eq!(
                    node.widget.end_datetime,
                    utils::add_utc_offset("2024-01-02 00:00:00")?
                );
            }
            _ => assert!(false),
        }

        Ok(())
    }
}
