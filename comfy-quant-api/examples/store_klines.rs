use comfy_quant_api::task::{BinanceKlinesTask, Task};
use sqlx::postgres::PgPool;
use std::sync::Arc;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let db_pool = Arc::new(
        PgPool::connect("postgres://postgres:postgres@localhost:5432/comfy_quant_dev")
            .await
            .unwrap(),
    );

    let task = BinanceKlinesTask::builder()
        .db_pool(db_pool)
        .market("binance")
        .symbol("BTCUSDT")
        .interval("1d")
        .start_time(1502928000)
        .end_time(1503705600)
        .build();

    let status_rx = task.run().await?;

    while let Ok(status) = status_rx.recv_async().await {
        println!("task status: {:?}", status);
    }

    println!("task finished");

    Ok(())
}
