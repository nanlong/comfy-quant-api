use crate::task_core::{status::TaskStatus, traits::Executable};
use anyhow::Result;
use async_stream::stream;
use bon::{bon, Builder};
use chrono::{DateTime, Utc};
use comfy_quant_database::kline::{self, Kline};
use comfy_quant_exchange::kline_stream::{calc_time_range_kline_count, BinanceKline};
use futures::{stream::BoxStream, StreamExt};
use sqlx::PgPool;
use std::sync::Arc;

const EXCHANGE: &str = "binance";

#[derive(Builder, Clone, Debug)]
#[builder(on(String, into))]
struct TaskParams {
    market: String,       // 市场
    symbol: String,       // 交易对
    interval: String,     // 时间间隔
    start_timestamp: i64, // 开始时间
    end_timestamp: i64,   // 结束时间
}

pub struct BinanceKlinesTask {
    db: Arc<PgPool>,
    params: TaskParams,
}

#[bon]
impl BinanceKlinesTask {
    #[builder(on(String, into))]
    pub fn new(
        db: Arc<PgPool>,
        market: String,
        symbol: String,
        interval: String,
        start_timestamp: i64,
        end_timestamp: i64,
    ) -> Self {
        let params = TaskParams::builder()
            .market(market)
            .symbol(symbol)
            .interval(interval)
            .start_timestamp(start_timestamp)
            .end_timestamp(end_timestamp)
            .build();

        BinanceKlinesTask { db, params }
    }
}

impl Executable for BinanceKlinesTask {
    type Output = BoxStream<'static, Result<TaskStatus<Kline>>>;

    async fn check_data_complete(&self) -> Result<bool> {
        let start_datetime = DateTime::<Utc>::from_timestamp(self.params.start_timestamp, 0)
            .ok_or_else(|| anyhow::anyhow!("start_timestamp is invalid"))?;
        let end_datetime = DateTime::<Utc>::from_timestamp(self.params.end_timestamp, 0)
            .ok_or_else(|| anyhow::anyhow!("end_timestamp is invalid"))?;

        let store_kline_count = kline::time_range_klines_count(
            &self.db,
            "binance",
            &self.params.market,
            &self.params.symbol,
            &self.params.interval,
            &start_datetime,
            &end_datetime,
        )
        .await?;

        let kline_count_expect = calc_time_range_kline_count(
            &self.params.interval,
            self.params.start_timestamp,
            self.params.end_timestamp,
        );

        Ok(store_kline_count == kline_count_expect)
    }

    async fn execute(&self) -> Result<Self::Output> {
        let is_data_complete = self.check_data_complete().await?;
        let params = self.params.clone();
        let db = Arc::clone(&self.db);

        let stream = stream! {
            yield Ok(TaskStatus::Initializing);

            if is_data_complete {
                // let mut klines_stream = kline::time_range_klines_stream(
                //     &db,
                //     EXCHANGE,
                //     &params.market,
                //     &params.symbol,
                //     &params.interval,
                //     params.start_timestamp * 1000,
                //     params.end_timestamp * 1000,
                // );

                // while let Some(Ok(kline)) = klines_stream.next().await {
                //     yield Ok(TaskStatus::Running(kline));
                // }
                yield Ok(TaskStatus::Finished);
                return;
            } else {
                let client = BinanceKline::default();

                let mut klines_stream = client.klines_stream(
                    &params.market,
                    &params.symbol,
                    &params.interval,
                    params.start_timestamp,
                    params.end_timestamp,
                );

                while let Some(kline_summary) = klines_stream.next().await {
                    let kline_summary = kline_summary?;
                    let open_time = DateTime::<Utc>::from_timestamp_millis(kline_summary.open_time).unwrap_or_default();

                    let kline_data = kline::Kline {
                        exchange: EXCHANGE.to_string(),
                        market: params.market.clone(),
                        symbol: params.symbol.clone(),
                        interval: params.interval.clone(),
                        open_time,
                        open_price: kline_summary.open.parse()?,
                        high_price: kline_summary.high.parse()?,
                        low_price: kline_summary.low.parse()?,
                        close_price: kline_summary.close.parse()?,
                        volume: kline_summary.volume.parse()?,
                        ..Default::default()
                    };

                    let kline = kline::create_or_update(&db, &kline_data).await?;

                    yield Ok(TaskStatus::Running(kline));
                }
            }

            yield Ok(TaskStatus::Finished);
        };

        Ok(Box::pin(stream))
    }
}
