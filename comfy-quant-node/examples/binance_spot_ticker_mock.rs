use anyhow::Result;
use chrono::{DateTime, Utc};
use comfy_quant_node::{
    base::{
        traits::node::{NodeConnector, NodeExecutor, NodePorts},
        Ports,
    },
    data::{SpotPairInfo, TickStream},
    exchange::{binance_spot_ticker_mock, BinanceSpotTickerMock},
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

            println!("tick_stream.next().await is done");

            #[allow(unreachable_code)]
            Ok::<(), anyhow::Error>(())
        });

        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    tracing::info!("start");

    // 2024-10-10T15:18:42.740182Z convert to DateTime<Utc>
    let start_datetime =
        DateTime::parse_from_rfc3339("2024-10-10T15:18:42.740182Z")?.with_timezone(&Utc);
    // 2024-10-10T16:18:42.740230Z convert to DateTime<Utc>
    let end_datetime =
        DateTime::parse_from_rfc3339("2024-10-10T16:18:42.740230Z")?.with_timezone(&Utc);

    // let start_datetime = Utc::now() - Duration::from_secs(60 * 60);
    // let end_datetime = Utc::now();

    let widget = binance_spot_ticker_mock::Widget::builder()
        .base_currency("BTC")
        .quote_currency("USDT")
        .start_datetime(start_datetime)
        .end_datetime(end_datetime)
        .build();

    dbg!(&widget);

    let mut node1 = BinanceSpotTickerMock::try_new(widget)?;
    let mut node2 = DebugNode::new();

    node1.connection::<SpotPairInfo>(&mut node2, 0, 0).await?;
    node1.connection::<TickStream>(&mut node2, 1, 1).await?;

    println!("node2.execute()");
    node2.execute().await?;

    println!("node1.execute()");
    node1.execute().await?;

    // println!("finished");
    sleep(Duration::from_secs(10000)).await;

    Ok(())
}
