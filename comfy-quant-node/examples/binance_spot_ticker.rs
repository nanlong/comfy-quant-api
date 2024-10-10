use anyhow::Result;
use comfy_quant_node::{
    base::{
        traits::node::{NodeConnector, NodeExecutor, NodePorts},
        Ports,
    },
    data::{SpotPairInfo, Tick, TickStream},
    exchange::{binance_spot_ticker, BinanceSpotTicker},
};
use futures::StreamExt;
use std::{sync::Arc, time::Duration};
use tokio::time::sleep;

struct DebugNode {
    ports: Ports,
}

impl DebugNode {
    pub fn new() -> Self {
        let ports = Ports::new();
        Self { ports }
    }
}

impl NodePorts for DebugNode {
    fn get_ports(&self) -> Result<&Ports> {
        Ok(&self.ports)
    }

    fn get_ports_mut(&mut self) -> Result<&mut Ports> {
        Ok(&mut self.ports)
    }
}

impl NodeExecutor for DebugNode {
    async fn execute(&mut self) -> Result<()> {
        let slot = self.ports.get_input::<SpotPairInfo>(0)?;
        let pair_info = slot.data();
        dbg!(&pair_info);

        let mut slot = self.ports.get_input::<TickStream>(1)?;

        tokio::spawn(async move {
            let tick_stream = Arc::make_mut(&mut slot);

            while let Some(tick) = tick_stream.next().await {
                dbg!(&tick);
            }

            #[allow(unreachable_code)]
            Ok::<(), anyhow::Error>(())
        });

        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let widget = binance_spot_ticker::Widget::builder()
        .base_currency("BTC")
        .quote_currency("USDT")
        .build();
    let mut node1 = BinanceSpotTicker::try_new(widget)?;
    let mut node2 = DebugNode::new();

    node1.connection::<SpotPairInfo>(&mut node2, 0, 0).await?;
    node1.connection::<Tick>(&mut node2, 1, 1).await?;

    println!("node2.execute()");
    node2.execute().await?;

    println!("node1.execute()");
    node1.execute().await?;

    // println!("finished");
    sleep(Duration::from_secs(10000)).await;

    Ok(())
}
