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

    // 2017-08-17 08:00:00 - 2017-08-26 08:00:00
    let task = BinanceKlinesTask::builder()
        .db_pool(db_pool)
        .market("spot")
        .symbol("BTCUSDT")
        .interval("1s")
        .start_time_second(1704081600)
        .end_time_second(1704168000)
        .build();

    let status_rx = task.run().await?;

    while let Ok(status) = status_rx.recv_async().await {
        println!("task status: {:?}", status);
    }

    println!("task finished");

    Ok(())
}
