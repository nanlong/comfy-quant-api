use super::{status::TaskStatus, traits::Task};
use anyhow::Result;
use bon::{bon, Builder};
use comfy_quant_client::kline::BinanceKline;
use comfy_quant_data::kline;
use flume::Receiver;
use futures::StreamExt;
use rust_decimal::Decimal;
use sqlx::PgPool;
use std::sync::Arc;

#[derive(Builder, Clone)]
#[builder(on(String, into))]
struct TaskParams {
    market: String,   // 市场
    symbol: String,   // 交易对
    interval: String, // 时间间隔
    start_time: u64,  // 开始时间
    end_time: u64,    // 结束时间
}

pub struct BinanceKlinesTask {
    db_pool: Arc<PgPool>,
    params: TaskParams,
}

#[bon]
impl BinanceKlinesTask {
    #[builder]
    pub fn new(
        db_pool: Arc<PgPool>,
        market: impl Into<String>,
        symbol: impl Into<String>,
        interval: impl Into<String>,
        start_time: u64,
        end_time: u64,
    ) -> Self {
        let params = TaskParams::builder()
            .market(market)
            .symbol(symbol)
            .interval(interval)
            .start_time(start_time)
            .end_time(end_time)
            .build();

        Self { db_pool, params }
    }
}

impl Task for BinanceKlinesTask {
    async fn run(self) -> Result<Receiver<TaskStatus>> {
        let BinanceKlinesTask { params, db_pool } = self;
        let client = Arc::new(BinanceKline::new());
        let (tx, rx) = flume::bounded::<TaskStatus>(1);

        tokio::spawn(async move {
            let result = async {
                tx.send_async(TaskStatus::Running).await?;

                let mut klines_stream = client.klines_stream(
                    &params.market,
                    &params.symbol,
                    &params.interval,
                    params.start_time,
                    params.end_time,
                );

                while let Some(kline_summary) = klines_stream.next().await {
                    match kline_summary {
                        Ok(kline_summary) => {
                            let kline_data = kline::Kline {
                                exchange: "binance".to_string(),
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

                            let result = kline::insert_or_update(&db_pool, &kline_data).await;

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
