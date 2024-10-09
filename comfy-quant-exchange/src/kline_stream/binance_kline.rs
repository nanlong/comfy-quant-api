use super::utils::calc_time_range_group;
use crate::BinanceClient;
use anyhow::Result;
use async_stream::stream;
use binance::model::{KlineSummaries, KlineSummary};
use futures::Stream;
use std::{str::FromStr, sync::Arc};

const KLINE_LIMIT: u16 = 1000;

#[allow(unused)]
#[derive(Debug, Clone)]
pub enum Market {
    Spot,
    Futures,
}

impl FromStr for Market {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "spot" => Ok(Market::Spot),
            "futures" => Ok(Market::Futures),
            _ => Err(anyhow::anyhow!("Invalid market: {}", s)),
        }
    }
}

#[derive(Debug)]
pub struct BinanceKline {
    client: BinanceClient,
}

impl Default for BinanceKline {
    fn default() -> Self {
        Self::new()
    }
}

#[allow(unused)]
impl BinanceKline {
    pub fn new() -> Self {
        let client = BinanceClient::builder().build();
        BinanceKline { client }
    }

    // 获取K线流
    pub fn klines_stream(
        &self,
        market: impl Into<String>,   // 市场
        symbol: impl Into<String>,   // 交易对
        interval: impl Into<String>, // 时间间隔
        start_time: i64,             // 开始时间
        end_time: i64,               // 结束时间
    ) -> impl Stream<Item = Result<KlineSummary>> {
        let market = market.into();
        let symbol = symbol.into();
        let interval = interval.into();
        let client = self.client.clone();
        let (tx, rx) = flume::bounded(1);
        let (error_tx, error_rx) = flume::bounded(1);
        let semaphore = Arc::new(async_lock::Semaphore::new(1));
        let time_range_groups = calc_time_range_group(&interval, start_time, end_time, KLINE_LIMIT);

        // 使用 tokio::spawn 会有问题，所以使用 tokio::task::spawn_blocking
        // 报错信息: Cannot drop a runtime in a context where blocking is not allowed. This happens when a runtime is dropped from within an asynchronous context.
        // 原因: reqwest 的 runtime 在异步上下文中被释放了
        tokio::task::spawn_blocking(move || {
            let result = (move || {
                let market = market.parse::<Market>()?;

                for (start_time, end_time) in time_range_groups {
                    let KlineSummaries::AllKlineSummaries(klines) = match market {
                        Market::Spot => client.spot().get_klines(
                            &symbol,
                            &interval,
                            KLINE_LIMIT,
                            start_time as u64,
                            end_time as u64,
                        )?,
                        Market::Futures => client.futures().get_klines(
                            &symbol,
                            &interval,
                            KLINE_LIMIT,
                            start_time as u64,
                            end_time as u64,
                        )?,
                    };

                    for kline in klines {
                        let guard = semaphore.acquire_arc_blocking();
                        tx.send((kline, guard))?;
                    }
                }

                Ok(())
            })();

            if let Err(e) = result {
                error_tx.send(e)?;
            }

            Ok::<(), anyhow::Error>(())
        });

        let kline_stream = stream! {
            loop {
                tokio::select! {
                    Ok((kline, _guard)) = rx.recv_async() => {
                        yield Ok(kline);
                    }
                    Ok(err) = error_rx.recv_async() => {
                        yield Err(err);
                        break;
                    }
                    else => break,
                }
            }
        };

        Box::pin(kline_stream)
    }
}
