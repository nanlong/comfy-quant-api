use comfy_quant_base::{KlineInterval, Market, Symbol};
use comfy_quant_exchange::kline_stream::BinanceKline;
use futures::StreamExt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    let client = BinanceKline::default();

    let market = Market::Spot;
    let symbol = Symbol::new("BTCUSDT");
    let interval = KlineInterval::TwelveHours;

    // 2024-10-10 15:18:42 2024-10-10 16:18:42
    let mut klines_stream =
        client.klines_stream(&market, &symbol, &interval, 1502928000, 1503705600);

    while let Some(kline) = klines_stream.next().await {
        println!("{:?}", kline);
    }

    Ok(())
}
