use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Link {
    pub link_id: u32,
    pub origin_id: u32,
    pub origin_slot: u32,
    pub target_id: u32,
    pub target_slot: u32,
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

#[derive(Serialize, Deserialize, Debug)]
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

#[derive(Serialize, Deserialize, Debug)]
pub struct Input {
    name: String,
    #[serde(rename = "type")]
    input_type: String,
    link: Option<u32>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Output {
    name: String,
    #[serde(rename = "type")]
    output_type: String,
    links: Option<Vec<u32>>,
    slot_index: u32,
}

#[derive(Serialize, Deserialize, Debug)]
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
        let json_str = r#"[1, 2, 0, 5, 0, "tickerStream"]"#;
        let link: Link = serde_json::from_str(json_str)?;

        assert_eq!(link.link_id, 1);
        assert_eq!(link.origin_id, 2);
        assert_eq!(link.origin_slot, 0);
        assert_eq!(link.target_id, 5);
        assert_eq!(link.target_slot, 0);
        assert_eq!(link.link_type, "tickerStream");

        Ok(())
    }

    #[test]
    fn test_ticker_node_deserialize() -> anyhow::Result<()> {
        let json_str = r#"{"id":1,"type":"加密货币交易所/币安现货(Ticker)","pos":[118,183],"size":[210,102],"flags":{},"order":0,"mode":0,"outputs":[{"name":"交易所信息","type":"exchangeData","links":null,"slot_index":0},{"name":"最新成交价格","type":"tickerStream","links":null,"slot_index":1}],"properties":{"type":"cryptoExchange.binanceSpotTicker","params":["BTC","USDT"]}}"#;

        let node: Node = serde_json::from_str(json_str)?;

        // assert_eq!(node.id, 1);
        assert_eq!(node.node_type, "加密货币交易所/币安现货(Ticker)");
        // assert_eq!(node.outputs.unwrap().len(), 2);

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

    // #[test]
    // fn test_workflow_deserialize() -> anyhow::Result<()> {
    //     let json_str = r#"{"last_node_id":4,"last_link_id":4,"nodes":[{"id":2,"type":"交易策略/网格(现货)","pos":[521,102],"size":{"0":210,"1":310},"flags":{},"order":3,"mode":0,"inputs":[{"name":"交易所信息","type":"exchangeData","link":1},{"name":"最新成交价格","type":"tickerStream","link":2},{"name":"账户","type":"account","link":3},{"name":"回测","type":"backtest","link":4}],"properties":{"type":"strategy.gridSpot","params":["arithmetic","","",8,"","","","",true]}},{"id":1,"type":"加密货币交易所/币安现货(Ticker)","pos":[199,74],"size":{"0":210,"1":102},"flags":{},"order":0,"mode":0,"outputs":[{"name":"交易所信息","type":"exchangeData","links":[1],"slot_index":0},{"name":"最新成交价格","type":"tickerStream","links":[2],"slot_index":1}],"properties":{"type":"cryptoExchange.binanceSpotTicker","params":["BTC","USDT"]}},{"id":3,"type":"账户/币安子账户","pos":[202,225],"size":{"0":210,"1":82},"flags":{},"order":1,"mode":0,"outputs":[{"name":"账户","type":"account","links":[3],"slot_index":0}],"properties":{}},{"id":4,"type":"工具/回测设置","pos":[200,353],"size":{"0":210,"1":82},"flags":{},"order":2,"mode":0,"outputs":[{"name":"回测","type":"backtest","links":[4],"slot_index":0}],"properties":{}}],"links":[[1,1,0,2,0,"exchangeData"],[2,1,1,2,1,"tickerStream"],[3,3,0,2,2,"account"],[4,4,0,2,3,"backtest"]],"groups":[],"config":{},"extra":{},"version":0.4}"#;

    //     let workflow: Workflow = serde_json::from_str(json_str)?;

    //     assert_eq!(workflow.nodes.len(), 4);
    //     assert_eq!(workflow.links.len(), 3);

    //     Ok(())
    // }
}
