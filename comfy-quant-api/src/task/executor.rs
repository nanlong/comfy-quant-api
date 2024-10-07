use super::Task;
use crate::{
    app_context::APP_CONTEXT,
    task::{BinanceKlinesTask, TaskStatus},
};
use anyhow::Result;
use flume::Receiver;
use std::sync::Arc;

pub async fn run_binance_klines_task(
    market: impl Into<String>,
    symbol: impl Into<String>,
    interval: impl Into<String>,
    start_timestamp: i64,
    end_timestamp: i64,
) -> Result<Receiver<TaskStatus>> {
    let task = BinanceKlinesTask::builder()
        .db_pool(Arc::clone(&APP_CONTEXT.db))
        .market(market)
        .symbol(symbol)
        .interval(interval)
        .start_timestamp(start_timestamp)
        .end_timestamp(end_timestamp)
        .build();

    task.run().await
}
