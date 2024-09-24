use std::time::Duration;

use anyhow::Result;
use comfy_quant_node::{
    data::Ticker,
    exchange::BinanceSpotTicker,
    traits::{NodeConnector, NodeDataPort, NodeExecutor},
    DataPorts,
};
use tokio::time::sleep;
use tokio_stream::wrappers::{BroadcastStream, ReceiverStream};

struct DebugNode {
    data_ports: DataPorts,
}

impl DebugNode {
    pub fn new() -> Self {
        let data_ports = DataPorts::new(10, 0);
        Self { data_ports }
    }
}

impl NodeDataPort for DebugNode {
    fn get_data_port(&self) -> Result<&DataPorts> {
        Ok(&self.data_ports)
    }

    fn get_data_port_mut(&mut self) -> Result<&mut DataPorts> {
        Ok(&mut self.data_ports)
    }
}

impl NodeExecutor for DebugNode {
    async fn execute(&mut self) -> Result<()> {
        println!("{:?}", self.data_ports);

        let rx = self.data_ports.get_input::<Ticker>(1)?;

        while let Ok(ticker) = rx.recv().await {
            println!("ticker: {:?}", ticker);
        }

        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let mut node1 = BinanceSpotTicker::try_new("BTC", "USDT")?;
    let mut node2 = DebugNode::new();

    node1.connection::<Ticker>(&mut node2, 1, 1).await?;

    println!("node2.execute()");
    node2.execute().await?;

    println!("node1.execute()");
    node1.execute().await?;

    println!("finished");
    sleep(Duration::from_secs(10000)).await;

    Ok(())
}
