use crate::{
    node_core::{Connectable, Executable, Setupable},
    node_io::{SpotPairInfo, TickStream},
    nodes::node_kind::NodeKind,
    utils::generate_workflow_id,
};
use anyhow::{anyhow, Result};
use async_lock::{Barrier, Mutex};
use comfy_quant_exchange::client::spot_client_kind::SpotClientKind;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::{collections::HashMap, sync::Arc};
use tokio_util::sync::CancellationToken;

#[derive(Serialize, Deserialize, Debug)]
pub struct Workflow {
    last_node_id: u32,
    last_link_id: u32,
    nodes: Vec<Node>,
    links: Vec<Link>,
    groups: Vec<String>,
    config: HashMap<String, String>,
    extra: HashMap<String, String>,
    version: f32,

    #[serde(skip)]
    context: Option<Arc<WorkflowContext>>,
    #[serde(skip)]
    cancel_token: CancellationToken,
    #[serde(skip)]
    deserialized_nodes: HashMap<u32, Arc<Mutex<NodeKind>>>,
}

impl Workflow {
    // 初始化上下文
    pub fn initialize(&mut self, db: PgPool) {
        let barrier = Barrier::new(self.nodes.len());

        self.context = Some(Arc::new(WorkflowContext::new(db, barrier)));
        self.cancel_token = CancellationToken::new();
    }

    // 按照 order 排序
    fn sorted_nodes(&self) -> Vec<&Node> {
        let mut nodes_vec = self.nodes.iter().collect::<Vec<_>>();
        nodes_vec.sort_by_key(|node| node.order);
        nodes_vec
    }

    // 建立连接
    async fn make_connection(
        &self,
        origin: &NodeKind,
        target: &mut NodeKind,
        link: &Link,
    ) -> Result<()> {
        match link.link_type.as_str() {
            "SpotPairInfo" => origin.connection::<SpotPairInfo>(
                &mut *target,
                link.origin_slot,
                link.target_slot,
            )?,
            "TickStream" => {
                origin.connection::<TickStream>(&mut *target, link.origin_slot, link.target_slot)?
            }
            "SpotClient" => origin.connection::<SpotClientKind>(
                &mut *target,
                link.origin_slot,
                link.target_slot,
            )?,
            _ => anyhow::bail!("Invalid link type: {}", link.link_type),
        }

        Ok(())
    }
}

impl Executable for Workflow {
    async fn execute(&mut self) -> Result<()> {
        // 获取上下文
        let context = self
            .context
            .as_ref()
            .ok_or_else(|| anyhow!("context not setup"))?;

        // 反序列化节点
        for node in &self.nodes {
            let node_id = node.id;
            let mut node_kind = NodeKind::try_from(node)?;

            // 为节点初始化上下文
            // context 内部字段都由 Arc 智能指针包裹，克隆的代价很小
            node_kind.setup_context(Arc::clone(context));

            // 存储反序列化节点
            self.deserialized_nodes
                .insert(node_id, Arc::new(Mutex::new(node_kind)));
        }

        tracing::info!("Workflow deserialized nodes");

        // 建立连接
        for link in &self.links {
            let origin_node = self
                .deserialized_nodes
                .get(&link.origin_id)
                .ok_or_else(|| anyhow::anyhow!("Origin node not found: {}", link.origin_id))?
                .lock_arc_blocking();

            let mut target_node = self
                .deserialized_nodes
                .get(&link.target_id)
                .ok_or_else(|| anyhow::anyhow!("Target node not found: {}", link.target_id))?
                .lock_arc_blocking();

            self.make_connection(&origin_node, &mut target_node, link)
                .await?;
        }

        tracing::info!("Workflow make connection");

        // 按顺序从前至后执行节点
        for node in self.sorted_nodes().into_iter() {
            let node_id = node.id;

            let node_kind = Arc::clone(
                self.deserialized_nodes
                    .get(&node_id)
                    .ok_or_else(|| anyhow::anyhow!("Node not found: {}", node.id))?,
            );

            let cancel_token = self.cancel_token.clone();

            // 在单独的线程中执行节点
            tokio::spawn(async move {
                tokio::select! {
                    _ = async {
                        let mut node = node_kind.lock().await;
                        node.execute().await?;
                        Ok::<(), anyhow::Error>(())
                    } => {
                        tracing::info!("Node {} finished", node_id);
                    },
                    _ = cancel_token.cancelled() => {
                        tracing::info!("Node {} cancelled", node_id);
                    }
                }
            });
        }

        tracing::info!("Workflow nodes execute");

        Ok(())
    }
}

impl Drop for Workflow {
    fn drop(&mut self) {
        self.cancel_token.cancel();
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Link {
    link_id: u32,
    origin_id: u32,
    origin_slot: usize,
    target_id: u32,
    target_slot: usize,
    link_type: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Node {
    id: u32,
    #[serde(rename = "type")]
    node_type: String,
    pos: [u32; 2],
    // size: HashMap<String, u32>,
    // flags: HashMap<String, String>,
    order: u32,
    mode: u32,
    inputs: Option<Vec<Input>>,
    outputs: Option<Vec<Output>>,
    pub(crate) properties: Properties,
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
    pub(crate) prop_type: String,
    pub(crate) params: Vec<serde_json::Value>,
}

#[allow(unused)]
#[derive(Debug)]
pub struct WorkflowContext {
    id: String,       // 工作流ID
    db: Arc<PgPool>,  // 数据库
    barrier: Barrier, // 屏障
}

impl WorkflowContext {
    fn new(db: PgPool, barrier: Barrier) -> Self {
        let id = generate_workflow_id();
        let db = Arc::new(db);

        Self { id, db, barrier }
    }

    pub(crate) fn db_cloned(&self) -> Arc<PgPool> {
        Arc::clone(&self.db)
    }

    pub(crate) async fn wait(&self) {
        self.barrier.wait().await;
    }
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
