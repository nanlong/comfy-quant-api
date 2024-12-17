use crate::task_core::{status::TaskStatus, traits::Executable};
use anyhow::Result;
use async_stream::stream;
use bon::{bon, Builder};
use comfy_quant_base::{
    millis_to_datetime, secs_to_datetime, Exchange, KlineInterval, Market, Symbol,
};
use comfy_quant_database::kline::{self, CreateKlineParams, Kline};
use comfy_quant_exchange::kline_stream::{calc_time_range_kline_count, BinanceKline};
use futures::{stream::BoxStream, StreamExt};
use rust_decimal::Decimal;
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
        market: Market,
        symbol: Symbol,
        interval: KlineInterval,
        start_timestamp: i64,
        end_timestamp: i64,
    ) -> Result<Self> {
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
            &Exchange::Binance,
            &self.params.market,
            &self.params.symbol,
            &self.params.interval,
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
                    &params.market,
                    &params.symbol,
                    &params.interval,
                    params.start_timestamp,
                    params.end_timestamp,
                );

                while let Some(kline_summary) = klines_stream.next().await {
                    let kline_summary = kline_summary?;
                    let open_time = millis_to_datetime(kline_summary.open_time)?;
                    let open_price = kline_summary.open.parse::<Decimal>()?;
                    let high_price = kline_summary.high.parse::<Decimal>()?;
                    let low_price = kline_summary.low.parse::<Decimal>()?;
                    let close_price = kline_summary.close.parse::<Decimal>()?;
                    let volume = kline_summary.volume.parse::<Decimal>()?;

                    let data = CreateKlineParams::builder()
                        .exchange(Exchange::Binance)
                        .market(params.market.clone())
                        .symbol(params.symbol.clone())
                        .interval(params.interval.clone())
                        .open_time(open_time)
                        .open_price(open_price)
                        .high_price(high_price)
                        .low_price(low_price)
                        .close_price(close_price)
                        .volume(volume)
                        .build();

                    let kline = kline::create_or_update(&db, data).await?;

                    yield Ok(TaskStatus::Running(kline));
                }
            }

            yield Ok(TaskStatus::Finished);
        };

        Ok(Box::pin(stream))
    }
}
