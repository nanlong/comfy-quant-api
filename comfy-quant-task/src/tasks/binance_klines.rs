use crate::task_core::{status::TaskStatus, traits::Executable};
use anyhow::Result;
use async_stream::stream;
use bon::{bon, Builder};
use comfy_quant_base::{millis_to_datetime, secs_to_datetime, KlineInterval, Market, Symbol};
use comfy_quant_database::kline::{self, Kline};
use comfy_quant_exchange::{
    client::spot_client::base::BINANCE_EXCHANGE_NAME,
    kline_stream::{calc_time_range_kline_count, BinanceKline},
};
use futures::{stream::BoxStream, StreamExt};
use sqlx::PgPool;
use std::sync::Arc;

#[derive(Builder, Clone, Debug)]
#[builder(on(_, into))]
struct TaskParams {
    market: Market,          // 市场
    symbol: Symbol,          // 交易对
    interval: KlineInterval, // 时间间隔
    start_timestamp: i64,    // 开始时间
    end_timestamp: i64,      // 结束时间
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
        market: &str,
        symbol: &str,
        interval: &str,
        start_timestamp: i64,
        end_timestamp: i64,
    ) -> Result<Self> {
        let interval: KlineInterval = interval.try_into()?;

        let params = TaskParams::builder()
            .market(market)
            .symbol(symbol)
            .interval(interval)
            .start_timestamp(start_timestamp)
            .end_timestamp(end_timestamp)
            .build();

        Ok(BinanceKlinesTask { db, params })
    }
}

impl Executable for BinanceKlinesTask {
    type Output = BoxStream<'static, Result<TaskStatus<Kline>>>;

    async fn check_data_complete(&self) -> Result<bool> {
        let start_datetime = secs_to_datetime(self.params.start_timestamp)?;
        let end_datetime = secs_to_datetime(self.params.end_timestamp)?;

        let store_kline_count = kline::time_range_klines_count(
            &self.db,
            "binance",
            self.params.market.as_ref(),
            self.params.symbol.as_ref(),
            self.params.interval.as_ref(),
            &start_datetime,
            &end_datetime,
        )
        .await?;

        let kline_count_expect = calc_time_range_kline_count(
            self.params.interval.as_ref(),
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
                //     BINANCE_EXCHANGE_NAME,
                //     &params.market,
                //     &params.symbol,
                //     &params.interval,
                //     &start_datetime,
                //     &end_datetime,
                // );


                // while let Some(Ok(kline)) = klines_stream.next().await {
                //     yield Ok(TaskStatus::Running(kline));
                // }

                yield Ok(TaskStatus::Finished);
                return;
            } else {
                let client = BinanceKline::default();

                let mut klines_stream = client.klines_stream(
                    params.market.clone(),
                    params.symbol.clone(),
                    params.interval.clone(),
                    params.start_timestamp,
                    params.end_timestamp,
                );

                while let Some(kline_summary) = klines_stream.next().await {
                    let kline_summary = kline_summary?;
                    let open_time = millis_to_datetime(kline_summary.open_time)?;

                    let kline_data = kline::Kline {
                        exchange: BINANCE_EXCHANGE_NAME.to_string(),
                        market: params.market.clone(),
                        symbol: params.symbol.to_string(),
                        interval: params.interval.to_string(),
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
