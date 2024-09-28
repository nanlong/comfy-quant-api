use anyhow::Result;
use comfy_quant_client::subscription::{BinanceSpotTicker, Subscription};
use sqlx::PgPool;

#[tokio::main]
async fn main() -> Result<()> {
    let pool =
        PgPool::connect("postgres://postgres:postgres@localhost:5432/comfy_quant_dev").await?;

    // let kline = get_kline(&pool, "binance", "BTCUSDT", "1m", 1717233600).await?;

    // println!("kline: {:?}", kline);

    let client = BinanceSpotTicker::new();

    let res = client.execute(&pool).await;

    println!("res: {:?}", res);

    tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;

    Ok(())
}
