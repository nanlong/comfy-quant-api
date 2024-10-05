use comfy_quant_client::history_klines::BinanceHistoryKlines;
use futures::StreamExt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = BinanceHistoryKlines::new();

    dbg!(&client);

    let mut klines_stream = client.klines_stream("spot", "BTCUSDT", "12h", 1502928000, 1503705600);

    while let Some(kline) = klines_stream.next().await {
        println!("{:?}", kline);
    }

    Ok(())
}
