use comfy_quant_exchange::kline_stream::BinanceKline;
use futures::StreamExt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = BinanceKline::new();

    let mut klines_stream = client.klines_stream("spot", "BTCUSDT", "12h", 1502928000, 1503705600);

    while let Some(kline) = klines_stream.next().await {
        println!("{:?}", kline);
    }

    Ok(())
}
