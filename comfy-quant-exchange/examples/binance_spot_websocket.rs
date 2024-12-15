use binance::config::Config;
use comfy_quant_exchange::exchange::binance::BinanceClient;
use futures::StreamExt;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = Config::default().set_ws_endpoint("wss://data-stream.binance.vision/ws");
    let client = BinanceClient::builder().config(config).build();
    let websocket = client.spot_websocket("btcusdt@aggTrade");

    let mut stream = websocket.subscribe().await?;

    println!("got stream");

    while let Some(event) = stream.next().await {
        println!("{:?}", event);
    }

    Ok(())
}
