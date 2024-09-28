use anyhow::Result;
use binance::model::DayTickerEvent;
use comfy_quant_client::subscription::TickerWrapper;
use sqlx::PgPool;

#[tokio::main]
async fn main() -> Result<()> {
    let pool =
        PgPool::connect("postgres://postgres:postgres@localhost:5432/comfy_quant_dev").await?;

    let ticker = TickerWrapper(DayTickerEvent {
        event_type: "24hrTicker".to_string(),
        event_time: 1727458847244,
        symbol: "ETHBTC".to_string(),
        price_change: "0.00050000".to_string(),
        price_change_percent: "1.238".to_string(),
        average_price: "0.04061583".to_string(),
        prev_close: "0.04040000".to_string(),
        current_close: "0.04090000".to_string(),
        current_close_qty: "0.05970000".to_string(),
        best_bid: "0.04089000".to_string(),
        best_bid_qty: "11.33200000".to_string(),
        best_ask: "0.04090000".to_string(),
        best_ask_qty: "30.76020000".to_string(),
        open: "0.04040000".to_string(),
        high: "0.04119000".to_string(),
        low: "0.04030000".to_string(),
        volume: "21129.44080000".to_string(),
        quote_volume: "858.18974131".to_string(),
        open_time: 1727372447244,
        close_time: 1727458847244,
        first_trade_id: 467845157,
        last_trade_id: 468018510,
        num_trades: 173354,
    });

    let kline = ticker.try_into_kline(&pool, "1m").await?;

    println!("kline: {:?}", kline);

    Ok(())
}
