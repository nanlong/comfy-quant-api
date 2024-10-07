use comfy_quant_api::task::executor::run_binance_klines_task;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let status_rx =
        run_binance_klines_task("spot", "BTCUSDT", "1m", 1704081600, 1704168000).await?;

    while let Ok(status) = status_rx.recv_async().await {
        println!("task status: {:?}", status);
    }

    println!("task finished");

    Ok(())
}
