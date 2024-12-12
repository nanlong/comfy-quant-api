use anyhow::Result;
use async_lock::RwLock;
use comfy_quant_node::{
    node_core::{ExchangeRateManager, NodeExecutable},
    workflow::Workflow,
};
use sqlx::postgres::PgPoolOptions;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let db = PgPoolOptions::new()
        .max_connections(20)
        .connect_lazy("postgres://postgres:postgres@localhost:5432/comfy_quant_dev")?;

    let json_str = r#"{"last_node_id":3,"last_link_id":3,"nodes":[{"id":2,"type":"加密货币交易所/币安现货(Ticker Mock)","pos":[210,58],"size":[240,150],"flags":{},"order":0,"mode":0,"outputs":[{"name":"现货交易对","type":"SpotPairInfo","links":[1],"slot_index":0},{"name":"Tick数据流","type":"TickStream","links":[2],"slot_index":1}],"properties":{"type":"data.BacktestSpotTicker","params":["BTC","USDT","2024-10-10 15:18:42","2024-10-10 16:18:42"]}},{"id":1,"type":"账户/币安账户(Mock)","pos":[224,295],"size":{"0":210,"1":106},"flags":{},"order":1,"mode":0,"outputs":[{"name":"现货账户客户端","type":"SpotClient","links":[3],"slot_index":0}],"properties":{"type":"client.BacktestSpotClient","params":[0.001, [["USDT",1000]]]}},{"id":3,"type":"交易策略/网格(现货)","pos":[520,93],"size":{"0":210,"1":290},"flags":{},"order":2,"mode":0,"inputs":[{"name":"现货交易对","type":"SpotPairInfo","link":1},{"name":"现货账户客户端","type":"SpotClient","link":3},{"name":"Tick数据流","type":"TickStream","link":2}],"properties":{"type":"strategy.SpotGrid","params":["arithmetic",60000,65000,8,1000,"","","",true]}}],"links":[[1,2,0,3,0,"SpotPairInfo"],[2,2,1,3,2,"TickStream"],[3,1,0,3,1,"SpotClient"]],"groups":[],"config":{},"extra":{},"version":0.4}"#;

    let mut workflow: Workflow = serde_json::from_str(json_str)?;

    workflow
        .setup(
            Arc::new(db),
            Arc::new(RwLock::new(ExchangeRateManager::default())),
            "USDT",
        )
        .await?;

    workflow.execute().await?;

    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

    dbg!(&workflow);

    // drop(workflow);
    // tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

    tokio::time::sleep(tokio::time::Duration::from_secs(100000)).await;
    Ok(())
}
