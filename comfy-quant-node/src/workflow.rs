use crate::{
    node_core::{ExchangeRate, ExchangeRateManager, NodeCoreExt, NodeExecutable, TradeStats},
    node_io::{SpotPairInfo, TickStream},
    nodes::node_kind::NodeKind,
};
use anyhow::{anyhow, Result};
use async_lock::RwLock;
use chrono::{DateTime, Utc};
use comfy_quant_base::{arc_rwlock, generate_workflow_id, vec_arc_rwlock};
use comfy_quant_exchange::{client::spot_client_kind::SpotClientKind, store::PriceStore};
use itertools::Itertools;
use rust_decimal::Decimal;
use serde::{ser::SerializeStruct, Deserialize, Serialize};
use sqlx::PgPool;
use std::{collections::HashMap, future::Future, sync::Arc, time::Instant};
use tokio_util::sync::CancellationToken;

#[derive(Deserialize, Debug)]
pub struct Workflow {
    last_node_id: u32,
    last_link_id: u32,
    nodes: Vec<Node>,
    links: Vec<Link>,
    groups: Vec<String>,
    config: HashMap<String, String>,
    extra: HashMap<String, String>,
    version: f32,
    #[serde(default, with = "arc_rwlock")]
    quote_asset: Arc<RwLock<QuoteAsset>>, // 报价资产, 默认USDT
    #[serde(default, with = "vec_arc_rwlock")]
    execution_history: Vec<Arc<RwLock<ExecutionRecord>>>, // 每一次的执行时间
    #[serde(default, with = "arc_rwlock")]
    running_time: Arc<RwLock<u128>>, // 运行持续时间(微妙)

    #[serde(skip)]
    deserialized_nodes: HashMap<u32, Arc<RwLock<NodeKind>>>, // 反序列化节点
    #[serde(skip)]
    context: Option<Arc<WorkflowContext>>, // 上下文
    #[serde(skip)]
    token: CancellationToken, // 取消令牌
}

impl Workflow {
    // 初始化上下文
    pub async fn setup(
        &mut self,
        db: Arc<PgPool>,                                         // 数据库
        exchange_rate_manager: Arc<RwLock<ExchangeRateManager>>, // 汇率管理器
        quote_asset: impl Into<QuoteAsset>,                      // 报价资产
    ) -> Result<()> {
        let quote_asset = Arc::new(RwLock::new(quote_asset.into()));
        let context = Arc::new(WorkflowContext::new(
            db,
            Arc::clone(&quote_asset),
            exchange_rate_manager,
            Arc::clone(&self.running_time),
        ));

        self.quote_asset = Arc::clone(&quote_asset);
        self.context = Some(Arc::clone(&context));

        for node in &mut self.nodes {
            node.context = Some(Arc::clone(&context));
        }

        // 反序列化节点
        for node in &self.nodes {
            let node_id = node.id;
            let mut node_kind = NodeKind::try_from(node.clone())?;

            node_kind.setup().await?;

            // 存储反序列化节点
            self.deserialized_nodes
                .insert(node_id, Arc::new(RwLock::new(node_kind)));
        }

        tracing::info!("Workflow deserialized nodes");

        // 建立连接
        for link in &self.links {
            let origin_node = self
                .deserialized_nodes
                .get(&link.origin_id)
                .ok_or_else(|| anyhow::anyhow!("Origin node not found: {}", link.origin_id))?
                .read()
                .await;

            let mut target_node = self
                .deserialized_nodes
                .get(&link.target_id)
                .ok_or_else(|| anyhow::anyhow!("Target node not found: {}", link.target_id))?
                .write()
                .await;

            self.make_connection(&origin_node, &mut target_node, link)?;
        }

        tracing::info!("Workflow make connection");

        Ok(())
    }

    pub async fn update_quote_asset(&mut self, quote_asset: impl Into<QuoteAsset>) -> Result<()> {
        *self.context()?.quote_asset.write().await = quote_asset.into();
        Ok(())
    }

    // 按照 order 排序
    fn sorted_nodes(&self) -> Vec<&Node> {
        let mut nodes_vec = self.nodes.iter().collect::<Vec<_>>();
        nodes_vec.sort_by_key(|node| node.order);
        nodes_vec
    }

    // 建立连接
    fn make_connection(&self, origin: &NodeKind, target: &mut NodeKind, link: &Link) -> Result<()> {
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

    fn context(&self) -> Result<&Arc<WorkflowContext>> {
        self.context
            .as_ref()
            .ok_or_else(|| anyhow!("Context not set"))
    }

    // 计算资产金额
    async fn calculate_asset_amount<F, Fut>(&self, f: F) -> Result<Decimal>
    where
        F: Fn(Arc<RwLock<NodeKind>>) -> Fut,
        Fut: Future<Output = Result<Decimal>>,
    {
        let mut value = Decimal::ZERO;

        for node in self.deserialized_nodes.values() {
            value += f(Arc::clone(node)).await?;
        }

        Ok(value)
    }
}

impl NodeExecutable for Workflow {
    async fn execute(&mut self) -> Result<()> {
        let start_at = Instant::now();
        let execute_time = Arc::new(RwLock::new(ExecutionRecord::new()));
        let cloned_execute_time = Arc::clone(&execute_time);
        let cloned_running_time = self.running_time.clone();
        let cloned_token = self.token.clone();
        let running_time = *cloned_running_time.read().await;

        self.execution_history.push(execute_time);

        // 计算运行时间
        tokio::spawn(async move {
            let update_times = || async {
                let mut execute_time_write = cloned_execute_time.write().await;
                let mut running_time_write = cloned_running_time.write().await;

                let elapsed = start_at.elapsed().as_micros();
                let time = running_time + elapsed;
                *running_time_write = time;
                execute_time_write.running_time = elapsed;
                execute_time_write.stop_at = Utc::now();
            };

            tokio::select! {
                _ = async {
                    loop {
                        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                        update_times().await;
                    }
                } => {}
                _ = cloned_token.cancelled() => {
                    update_times().await;
                }
            }
        });

        // 按顺序从前至后执行节点
        for node in self.sorted_nodes().into_iter() {
            let node_id = node.id;

            let mut node_kind = self
                .deserialized_nodes
                .get(&node_id)
                .ok_or_else(|| anyhow::anyhow!("Node not found: {:?}", node))?
                .write_arc()
                .await;

            let cloned_token = self.token.clone();

            // 在单独的线程中执行节点
            tokio::spawn(async move {
                tokio::select! {
                    _ = async {
                        node_kind.execute().await?;
                        Ok::<(), anyhow::Error>(())
                    } => {
                        tracing::info!("Node {:?} finished", node_kind);
                    },
                    _ = cloned_token.cancelled() => {
                        tracing::info!("Node {:?} cancelled", node_kind);
                    }
                }
            });
        }

        tracing::info!("Workflow nodes execute");

        Ok(())
    }
}

impl TradeStats for Workflow {
    // 初始资金
    async fn initial_capital(&self) -> Result<Decimal> {
        self.calculate_asset_amount(|node| async move { node.read().await.initial_capital().await })
            .await
    }

    // 已实现盈亏
    async fn realized_pnl(&self) -> Result<Decimal> {
        self.calculate_asset_amount(|node| async move { node.read().await.realized_pnl().await })
            .await
    }

    // 未实现盈亏
    async fn unrealized_pnl(&self) -> Result<Decimal> {
        self.calculate_asset_amount(|node| async move { node.read().await.unrealized_pnl().await })
            .await
    }

    // 运行时间
    async fn running_time(&self) -> Result<u128> {
        Ok(self.context()?.running_time().await)
    }
}

impl Serialize for Workflow {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        // 更新nodes
        let nodes = if self.deserialized_nodes.is_empty() {
            self.nodes
                .clone()
                .into_iter()
                .sorted_by_key(|node| node.order)
                .collect::<Vec<_>>()
        } else {
            self.deserialized_nodes
                .iter()
                .filter_map(|(_, node_kind)| Node::try_from(&*node_kind.read_blocking()).ok())
                .sorted_by_key(|node| node.order)
                .collect::<Vec<_>>()
        };

        let execution_history = self
            .execution_history
            .iter()
            .map(|execute_time| (*execute_time.read_blocking()).clone())
            .collect::<Vec<_>>();

        // 序列化所有非skip字段
        let mut state = serializer.serialize_struct("Workflow", 11)?;
        state.serialize_field("last_node_id", &self.last_node_id)?;
        state.serialize_field("last_link_id", &self.last_link_id)?;
        state.serialize_field("nodes", &nodes)?;
        state.serialize_field("links", &self.links)?;
        state.serialize_field("groups", &self.groups)?;
        state.serialize_field("config", &self.config)?;
        state.serialize_field("extra", &self.extra)?;
        state.serialize_field("version", &self.version)?;
        state.serialize_field("quote_asset", &*self.quote_asset.as_ref().read_blocking())?;
        state.serialize_field("execution_history", &execution_history)?;
        state.serialize_field("running_time", &*self.running_time.as_ref().read_blocking())?;

        state.end()
    }
}

impl Drop for Workflow {
    fn drop(&mut self) {
        self.token.cancel();
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct QuoteAsset(String);

impl QuoteAsset {
    pub fn new() -> Self {
        Self("USDT".to_string())
    }
}

impl Default for QuoteAsset {
    fn default() -> Self {
        Self::new()
    }
}

impl AsRef<str> for QuoteAsset {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl From<&str> for QuoteAsset {
    fn from(value: &str) -> Self {
        Self(value.to_string())
    }
}

impl From<String> for QuoteAsset {
    fn from(value: String) -> Self {
        Self(value)
    }
}

impl From<QuoteAsset> for String {
    fn from(value: QuoteAsset) -> Self {
        value.0
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
    pub id: u32,
    #[serde(rename = "type")]
    pub node_type: String,
    pub pos: [u32; 2],
    // size: HashMap<String, u32>,
    // flags: HashMap<String, String>,
    pub order: u32,
    pub mode: u32,
    pub inputs: Option<Vec<Input>>,
    pub outputs: Option<Vec<Output>>,
    pub properties: Properties,
    pub runtime_store: Option<String>,

    #[serde(skip)]
    pub context: Option<Arc<WorkflowContext>>, // 上下文
}

impl Node {
    pub(crate) fn workflow_context(&self) -> Result<&Arc<WorkflowContext>> {
        self.context
            .as_ref()
            .ok_or_else(|| anyhow!("Context not set"))
    }
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

// 记录工作流执行每次开始和结束的时间
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ExecutionRecord {
    start_at: DateTime<Utc>, // 开始时间
    stop_at: DateTime<Utc>,  // 结束时间
    running_time: u128,      // 运行持续时间(微妙)
}

impl ExecutionRecord {
    pub fn new() -> Self {
        Self {
            start_at: Utc::now(),
            stop_at: Utc::now(),
            running_time: 0,
        }
    }
}

impl Default for ExecutionRecord {
    fn default() -> Self {
        Self::new()
    }
}

#[allow(unused)]
#[derive(Debug)]
pub struct WorkflowContext {
    id: String,                                              // 工作流ID
    db: Arc<PgPool>,                                         // 数据库
    quote_asset: Arc<RwLock<QuoteAsset>>,                    // 计价货币
    price_store: Arc<RwLock<PriceStore>>,                    // 价格存储
    exchange_rate_manager: Arc<RwLock<ExchangeRateManager>>, // 汇率管理器
    running_time: Arc<RwLock<u128>>,                         // 运行持续时间(微妙)
}

#[allow(unused)]
impl WorkflowContext {
    pub(crate) fn new(
        db: Arc<PgPool>,
        quote_asset: Arc<RwLock<QuoteAsset>>,
        exchange_rate_manager: Arc<RwLock<ExchangeRateManager>>,
        running_time: Arc<RwLock<u128>>,
    ) -> Self {
        let id = generate_workflow_id();
        let price_store = Arc::new(RwLock::new(PriceStore::new()));

        Self {
            id,
            db,
            quote_asset,
            price_store,
            exchange_rate_manager,
            running_time,
        }
    }

    pub(crate) fn workflow_id(&self) -> &str {
        &self.id
    }

    pub(crate) fn cloned_db(&self) -> Arc<PgPool> {
        Arc::clone(&self.db)
    }

    pub(crate) fn cloned_price_store(&self) -> Arc<RwLock<PriceStore>> {
        Arc::clone(&self.price_store)
    }

    pub async fn exchange_rate(
        &self,
        base_asset: impl AsRef<str>,
        quote_asset: impl AsRef<str>,
    ) -> Result<ExchangeRate> {
        self.exchange_rate_manager
            .write()
            .await
            .get_rate(base_asset, quote_asset)
            .ok_or_else(|| anyhow!(""))
    }

    pub async fn running_time(&self) -> u128 {
        *self.running_time.read().await
    }

    pub async fn quote_asset(&self) -> QuoteAsset {
        let quote_asset = &*self.quote_asset.read().await;
        quote_asset.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn default_context(db: PgPool) -> Arc<WorkflowContext> {
        Arc::new(WorkflowContext::new(
            Arc::new(db),
            Arc::new(RwLock::new(QuoteAsset::new())),
            Arc::new(RwLock::new(ExchangeRateManager::default())),
            Arc::new(RwLock::new(0)),
        ))
    }

    #[test]
    fn test_link_deserialize() -> Result<()> {
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
    fn test_ticker_node_deserialize() -> Result<()> {
        let json_str = r#"{"id":1,"type":"加密货币交易所/币安现货(Ticker)","pos":[118,183],"size":[210,102],"flags":{},"order":0,"mode":0,"outputs":[{"name":"交易所信息","type":"exchangeData","links":null,"slot_index":0},{"name":"最新成交价格","type":"tickerStream","links":null,"slot_index":1}],"properties":{"type":"ExchangeInfo.binanceSpotTicker","params":["BTC","USDT"]}}"#;

        let node: Node = serde_json::from_str(json_str)?;

        assert_eq!(node.id, 1);
        assert_eq!(node.node_type, "加密货币交易所/币安现货(Ticker)");
        assert_eq!(node.outputs.unwrap().len(), 2);

        Ok(())
    }

    #[test]
    fn test_spot_grid_node_deserialize() -> Result<()> {
        let json_str = r#"{"id":1,"type":"交易策略/网格(现货)","pos":[329,146],"size":{"0":210,"1":310},"flags":{},"order":0,"mode":0,"inputs":[{"name":"交易所信息","type":"exchangeData","link":null},{"name":"最新成交价格","type":"tickerStream","link":null},{"name":"账户","type":"account","link":null},{"name":"回测","type":"backtest","link":null}],"properties":{"params":["arithmetic","","",8,"","","","",true]}}"#;

        let node: Node = serde_json::from_str(json_str)?;

        assert_eq!(node.id, 1);
        assert_eq!(node.node_type, "交易策略/网格(现货)");
        assert_eq!(node.inputs.unwrap().len(), 4);
        assert_eq!(node.properties.prop_type, "");

        Ok(())
    }

    #[sqlx::test]
    fn test_workflow_deserialize(db: PgPool) -> Result<()> {
        let json_str = r#"{"last_node_id":3,"last_link_id":3,"nodes":[{"id":2,"type":"加密货币交易所/币安现货(Ticker Mock)","pos":[210,58],"size":[240,150],"flags":{},"order":0,"mode":0,"outputs":[{"name":"现货交易对","type":"SpotPairInfo","links":[1],"slot_index":0},{"name":"Tick数据流","type":"TickStream","links":[2],"slot_index":1}],"properties":{"type":"data.BacktestSpotTicker","params":["BTC","USDT","2024-01-01 00:00:00","2024-01-02 00:00:00"]}},{"id":1,"type":"账户/币安账户(Mock)","pos":[224,295],"size":{"0":210,"1":106},"flags":{},"order":1,"mode":0,"outputs":[{"name":"现货账户客户端","type":"SpotClient","links":[3],"slot_index":0}],"properties":{"type":"client.BacktestSpotClient","params":[0.001, [["USDT",1000]]]}},{"id":3,"type":"交易策略/网格(现货)","pos":[520,93],"size":{"0":210,"1":290},"flags":{},"order":2,"mode":0,"inputs":[{"name":"现货交易对","type":"SpotPairInfo","link":1},{"name":"现货账户客户端","type":"SpotClient","link":3},{"name":"Tick数据流","type":"TickStream","link":2}],"properties":{"type":"strategy.SpotGrid","params":["arithmetic",1,1.1,8,1,"","","",true]}}],"links":[[1,2,0,3,0,"SpotPairInfo"],[2,2,1,3,2,"TickStream"],[3,1,0,3,1,"SpotClient"]],"groups":[],"config":{},"extra":{},"version":0.4}"#;

        let mut workflow: Workflow = serde_json::from_str(json_str)?;

        let exchange_rate_manager = Arc::new(RwLock::new(ExchangeRateManager::default()));

        workflow
            .setup(Arc::new(db), exchange_rate_manager, "USDT")
            .await?;

        assert_eq!(*workflow.running_time.as_ref().read_blocking(), 0);
        assert_eq!(workflow.nodes.len(), 3);
        assert_eq!(workflow.links.len(), 3);

        assert_eq!(serde_json::to_string(&workflow)?, "{\"last_node_id\":3,\"last_link_id\":3,\"nodes\":[{\"id\":2,\"type\":\"加密货币交易所/币安现货(Ticker Mock)\",\"pos\":[210,58],\"order\":0,\"mode\":0,\"inputs\":null,\"outputs\":[{\"name\":\"现货交易对\",\"type\":\"SpotPairInfo\",\"links\":[1],\"slot_index\":0},{\"name\":\"Tick数据流\",\"type\":\"TickStream\",\"links\":[2],\"slot_index\":1}],\"properties\":{\"type\":\"data.BacktestSpotTicker\",\"params\":[\"BTC\",\"USDT\",\"2024-01-01 00:00:00\",\"2024-01-02 00:00:00\"]},\"runtime_store\":null},{\"id\":1,\"type\":\"账户/币安账户(Mock)\",\"pos\":[224,295],\"order\":1,\"mode\":0,\"inputs\":null,\"outputs\":[{\"name\":\"现货账户客户端\",\"type\":\"SpotClient\",\"links\":[3],\"slot_index\":0}],\"properties\":{\"type\":\"client.BacktestSpotClient\",\"params\":[0.001,[[\"USDT\",1000]]]},\"runtime_store\":null},{\"id\":3,\"type\":\"交易策略/网格(现货)\",\"pos\":[520,93],\"order\":2,\"mode\":0,\"inputs\":[{\"name\":\"现货交易对\",\"type\":\"SpotPairInfo\",\"link\":1},{\"name\":\"现货账户客户端\",\"type\":\"SpotClient\",\"link\":3},{\"name\":\"Tick数据流\",\"type\":\"TickStream\",\"link\":2}],\"outputs\":null,\"properties\":{\"type\":\"strategy.SpotGrid\",\"params\":[\"arithmetic\",1,1.1,8,1,\"\",\"\",\"\",true]},\"runtime_store\":\"{\\\"stats\\\":{\\\"data\\\":{}},\\\"grid\\\":null,\\\"initialized\\\":false}\"}],\"links\":[{\"link_id\":1,\"origin_id\":2,\"origin_slot\":0,\"target_id\":3,\"target_slot\":0,\"link_type\":\"SpotPairInfo\"},{\"link_id\":2,\"origin_id\":2,\"origin_slot\":1,\"target_id\":3,\"target_slot\":2,\"link_type\":\"TickStream\"},{\"link_id\":3,\"origin_id\":1,\"origin_slot\":0,\"target_id\":3,\"target_slot\":1,\"link_type\":\"SpotClient\"}],\"groups\":[],\"config\":{},\"extra\":{},\"version\":0.4,\"quote_asset\":\"USDT\",\"execution_history\":[],\"running_time\":0}");

        let json_str = r#"{"running_time":100000,"last_node_id":3,"last_link_id":3,"nodes":[],"links":[],"groups":[],"config":{},"extra":{},"version":0.4}"#;

        let workflow: Workflow = serde_json::from_str(json_str)?;

        assert_eq!(*workflow.running_time.as_ref().read_blocking(), 100000);

        assert_eq!(serde_json::to_string(&workflow)?, "{\"last_node_id\":3,\"last_link_id\":3,\"nodes\":[],\"links\":[],\"groups\":[],\"config\":{},\"extra\":{},\"version\":0.4,\"quote_asset\":\"USDT\",\"execution_history\":[],\"running_time\":100000}");

        Ok(())
    }

    #[sqlx::test]
    async fn test_workflow_context(db: PgPool) {
        let context = default_context(db);
        assert_eq!(context.workflow_id().len(), 21);

        let db = context.cloned_db();
        assert_eq!(Arc::strong_count(&db), 2);
    }
}
