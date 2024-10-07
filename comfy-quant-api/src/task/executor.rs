use crate::task::TaskStatus;
use anyhow::Result;
use flume::Receiver;

#[allow(unused)]
pub async fn run_binance_spot_klines_task(
    symbol: impl Into<String>,
    interval: impl Into<String>,
    start_time_second: i64,
    end_time_second: i64,
) -> Result<Receiver<TaskStatus>> {
    // let task = BinanceKlinesTask::builder()
    //     .market("spot")
    //     .symbol(symbol)
    //     .interval(interval)
    //     .start_time_second(start_time_second)
    //     .end_time_second(end_time_second)
    //     .build();

    todo!()
}
