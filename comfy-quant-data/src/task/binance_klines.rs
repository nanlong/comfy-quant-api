use super::{status::TaskStatus, traits::Task};
use crate::kline;
use anyhow::Result;
use bon::{bon, Builder};
use comfy_quant_client::kline::{calc_time_range_kline_count, BinanceKline};
use flume::Receiver;
use futures::StreamExt;
use rust_decimal::Decimal;
use sqlx::PgPool;
use std::sync::Arc;

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
    #[builder]
    pub fn new(
        db: Arc<PgPool>,
        market: impl Into<String>,
        symbol: impl Into<String>,
        interval: impl Into<String>,
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

        Self { db, params }
    }
}

impl Task for BinanceKlinesTask {
    async fn check_data_complete(&self) -> Result<bool> {
        let store_kline_count = kline::time_range_klines_count(
            &self.db,
            "binance",
            &self.params.market,
            &self.params.symbol,
            &self.params.interval,
            self.params.start_timestamp * 1000,
            self.params.end_timestamp * 1000,
        )
        .await?;

        let kline_count_expect = calc_time_range_kline_count(
            &self.params.interval,
            self.params.start_timestamp,
            self.params.end_timestamp,
        );

        Ok(store_kline_count == kline_count_expect)
    }

    async fn run(self) -> Result<Receiver<TaskStatus>> {
        let is_data_complete = self.check_data_complete().await?;
        let BinanceKlinesTask { params, db } = self;
        let client = Arc::new(BinanceKline::new());
        let (tx, rx) = flume::bounded::<TaskStatus>(1);

        tokio::spawn(async move {
            let result = async {
                tx.send_async(TaskStatus::Running).await?;

                if is_data_complete {
                    tx.send_async(TaskStatus::Finished).await?;
                    return Ok::<(), anyhow::Error>(());
                }

                let mut klines_stream = client.klines_stream(
                    &params.market,
                    &params.symbol,
                    &params.interval,
                    params.start_timestamp,
                    params.end_timestamp,
                );

                while let Some(kline_summary) = klines_stream.next().await {
                    match kline_summary {
                        Ok(kline_summary) => {
                            let kline_data = kline::Kline {
                                exchange: "binance".to_string(),
                                market: params.market.clone(),
                                symbol: params.symbol.clone(),
                                interval: params.interval.clone(),
                                open_time: kline_summary.open_time,
                                open_price: kline_summary.open.parse::<Decimal>()?,
                                high_price: kline_summary.high.parse::<Decimal>()?,
                                low_price: kline_summary.low.parse::<Decimal>()?,
                                close_price: kline_summary.close.parse::<Decimal>()?,
                                volume: kline_summary.volume.parse::<Decimal>()?,
                                ..Default::default()
                            };

                            let result = kline::insert_or_update(&db, &kline_data).await;

                            if let Err(e) = result {
                                tx.send_async(TaskStatus::Failed(e.to_string())).await?;
                            }
                        }
                        Err(e) => {
                            tx.send_async(TaskStatus::Failed(e.to_string())).await?;
                        }
                    }
                }

                Ok::<(), anyhow::Error>(())
            }
            .await;

            match result {
                Ok(()) => tx.send_async(TaskStatus::Finished).await?,
                Err(e) => tx.send_async(TaskStatus::Failed(e.to_string())).await?,
            };

            Ok::<(), anyhow::Error>(())
        });

        Ok(rx)
    }
}
