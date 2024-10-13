use crate::{
    base::traits::{NodeConnector, NodeExecutor},
    data::{SpotPairInfo, TickStream},
    node_kind::NodeKind,
};
use anyhow::Result;
use comfy_quant_exchange::client::SpotClientKind;
use serde::{Deserialize, Serialize};
use std::{cell::RefCell, collections::HashMap};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Link {
    pub link_id: u32,
    pub origin_id: u32,
    pub origin_slot: usize,
    pub target_id: u32,
    pub target_slot: usize,
    pub link_type: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Workflow {
    // last_node_id: u32,
    // last_link_id: u32,
    pub nodes: Vec<Node>,
    pub links: Vec<Link>,
    // groups: Vec<String>,
    // config: HashMap<String, String>,
    // extra: HashMap<String, String>,
    // version: f32,
}

impl Workflow {
    pub async fn execute(&self) -> Result<()> {
        let mut deserialized_nodes = HashMap::new();

        // 反序列化节点
        for node in &self.nodes {
            let node_kind = NodeKind::try_from((*node).clone())?;
            deserialized_nodes.insert(node.id, RefCell::new(node_kind));
        }

        // 建立连接
        for link in &self.links {
            let origin_node = deserialized_nodes
                .get(&link.origin_id)
                .ok_or_else(|| anyhow::anyhow!("Origin node not found: {}", link.origin_id))?;

            let target_node = deserialized_nodes
                .get(&link.target_id)
                .ok_or_else(|| anyhow::anyhow!("Target node not found: {}", link.target_id))?;

            self.make_connection(&origin_node, &target_node, &link)?;
        }

        // 执行节点
        for node in self.sorted_nodes().iter().rev() {
            deserialized_nodes
                .get(&node.id)
                .ok_or_else(|| anyhow::anyhow!("Node not found: {}", node.id))?
                .borrow_mut()
                .execute()
                .await?;
        }

        Ok(())
    }

    // 按照 order 排序
    fn sorted_nodes(&self) -> Vec<&Node> {
        let mut nodes_vec = self.nodes.iter().collect::<Vec<_>>();
        nodes_vec.sort_by_key(|node| node.order);
        nodes_vec
    }

    // 建立连接
    fn make_connection(
        &self,
        origin_node: &RefCell<NodeKind>,
        target_node: &RefCell<NodeKind>,
        link: &Link,
    ) -> Result<()> {
        let origin = origin_node.borrow();
        let target = &mut *target_node.borrow_mut();

        match link.link_type.as_str() {
            "SpotPairInfo" => {
                origin.connection::<SpotPairInfo>(target, link.origin_slot, link.target_slot)?
            }
            "TickStream" => {
                origin.connection::<TickStream>(target, link.origin_slot, link.target_slot)?
            }
            "SpotClient" => {
                origin.connection::<SpotClientKind>(target, link.origin_slot, link.target_slot)?
            }
            _ => anyhow::bail!("Invalid link type: {}", link.link_type),
        }

        Ok(())
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Node {
    pub id: u32,
    #[serde(rename = "type")]
    pub node_type: String,
    pub pos: [u32; 2],
    // pub size: HashMap<String, u32>,
    // pub flags: HashMap<String, String>,
    pub order: u32,
    pub mode: u32,
    pub inputs: Option<Vec<Input>>,
    pub outputs: Option<Vec<Output>>,
    pub properties: Properties,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Input {
    name: String,
    #[serde(rename = "type")]
    input_type: String,
    link: Option<u32>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Output {
    name: String,
    #[serde(rename = "type")]
    output_type: String,
    links: Option<Vec<u32>>,
    slot_index: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Properties {
    #[serde(rename = "type", default)]
    pub prop_type: String,
    pub params: Vec<serde_json::Value>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_link_deserialize() -> anyhow::Result<()> {
        let json_str = r#"[1, 2, 0, 5, 0, "tickStream"]"#;
        let link: Link = serde_json::from_str(json_str)?;

        assert_eq!(link.link_id, 1);
        assert_eq!(link.origin_id, 2);
        assert_eq!(link.origin_slot, 0);
        assert_eq!(link.target_id, 5);
        assert_eq!(link.target_slot, 0);
        assert_eq!(link.link_type, "tickStream");

        Ok(())
    }

    #[test]
    fn test_ticker_node_deserialize() -> anyhow::Result<()> {
        let json_str = r#"{"id":1,"type":"加密货币交易所/币安现货(Ticker)","pos":[118,183],"size":[210,102],"flags":{},"order":0,"mode":0,"outputs":[{"name":"交易所信息","type":"exchangeData","links":null,"slot_index":0},{"name":"最新成交价格","type":"tickerStream","links":null,"slot_index":1}],"properties":{"type":"ExchangeInfo.binanceSpotTicker","params":["BTC","USDT"]}}"#;

        let node: Node = serde_json::from_str(json_str)?;

        assert_eq!(node.id, 1);
        assert_eq!(node.node_type, "加密货币交易所/币安现货(Ticker)");
        assert_eq!(node.outputs.unwrap().len(), 2);

        Ok(())
    }

    #[test]
    fn test_spot_grid_node_deserialize() -> anyhow::Result<()> {
        let json_str = r#"{"id":1,"type":"交易策略/网格(现货)","pos":[329,146],"size":{"0":210,"1":310},"flags":{},"order":0,"mode":0,"inputs":[{"name":"交易所信息","type":"exchangeData","link":null},{"name":"最新成交价格","type":"tickerStream","link":null},{"name":"账户","type":"account","link":null},{"name":"回测","type":"backtest","link":null}],"properties":{"params":["arithmetic","","",8,"","","","",true]}}"#;

        let node: Node = serde_json::from_str(json_str)?;

        assert_eq!(node.id, 1);
        assert_eq!(node.node_type, "交易策略/网格(现货)");
        assert_eq!(node.inputs.unwrap().len(), 4);
        assert_eq!(node.properties.prop_type, "");

        Ok(())
    }

    #[test]
    fn test_workflow_deserialize() -> anyhow::Result<()> {
        let json_str = r#"{"last_node_id":3,"last_link_id":3,"nodes":[{"id":2,"type":"加密货币交易所/币安现货(Ticker Mock)","pos":[210,58],"size":[240,150],"flags":{},"order":0,"mode":0,"outputs":[{"name":"现货交易对","type":"SpotPairInfo","links":[1],"slot_index":0},{"name":"Tick数据流","type":"TickStream","links":[2],"slot_index":1}],"properties":{"type":"cryptoExchange.binanceSpotTickerMock","params":["BTC","USDT","2024-01-01 00:00:00","2024-01-02 00:00:00"]}},{"id":1,"type":"账户/币安账户(Mock)","pos":[224,295],"size":{"0":210,"1":106},"flags":{},"order":1,"mode":0,"outputs":[{"name":"现货账户客户端","type":"SpotClient","links":[3],"slot_index":0}],"properties":{"type":"cryptoExchange.binanceSpotAccountMock","params":[0.001, [["USDT",1000]]]}},{"id":3,"type":"交易策略/网格(现货)","pos":[520,93],"size":{"0":210,"1":290},"flags":{},"order":2,"mode":0,"inputs":[{"name":"现货交易对","type":"SpotPairInfo","link":1},{"name":"现货账户客户端","type":"SpotClient","link":3},{"name":"Tick数据流","type":"TickStream","link":2}],"properties":{"type":"strategy.gridSpot","params":["arithmetic","","",8,"","","","",true]}}],"links":[[1,2,0,3,0,"SpotPairInfo"],[2,2,1,3,2,"TickStream"],[3,1,0,3,1,"SpotClient"]],"groups":[],"config":{},"extra":{},"version":0.4}"#;

        let workflow: Workflow = serde_json::from_str(json_str)?;

        assert_eq!(workflow.nodes.len(), 3);
        assert_eq!(workflow.links.len(), 3);

        Ok(())
    }
}
