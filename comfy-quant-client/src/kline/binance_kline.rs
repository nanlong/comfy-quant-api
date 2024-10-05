use crate::BinanceClient;
use anyhow::Result;
use async_stream::stream;
use binance::model::{KlineSummaries, KlineSummary};
use futures::Stream;
use std::{str::FromStr, sync::Arc};

const KLINE_LIMIT: u64 = 1000;

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
        start_time: u64,             // 开始时间
        end_time: u64,               // 结束时间
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
                            KLINE_LIMIT as u16,
                            start_time,
                            end_time,
                        )?,
                        Market::Futures => client.futures().get_klines(
                            &symbol,
                            &interval,
                            KLINE_LIMIT as u16,
                            start_time,
                            end_time,
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

// 计算时间范围内的K线数量
fn calc_kline_count(interval: &str, start_time_second: u64, end_time_second: u64) -> u64 {
    let seconds = interval_to_seconds(interval);
    (end_time_second - start_time_second) / seconds + 1
}

// 计算时间范围内的K线分组
fn calc_time_range_group(
    interval: &str,
    start_time_second: u64,
    end_time_second: u64,
    limit: u64,
) -> Vec<(u64, u64)> {
    let mut result = Vec::new();
    let mut start_time = start_time_second;
    let interval_seconds = interval_to_seconds(interval);
    let kline_count = calc_kline_count(interval, start_time_second, end_time_second);

    for _i in 0..=(kline_count / limit) {
        let end_time = start_time + interval_seconds * limit;

        if end_time > end_time_second {
            result.push((start_time * 1000, end_time_second * 1000));
            break;
        } else {
            result.push((start_time * 1000, end_time * 1000 - 1));
            start_time = end_time;
        }
    }

    result
}

// 将时间间隔转换为秒
fn interval_to_seconds(interval: &str) -> u64 {
    match interval {
        "1s" => 1,
        "1m" => 60,
        "3m" => 180,
        "5m" => 300,
        "15m" => 900,
        "30m" => 1800,
        "1h" => 3600,
        "2h" => 7200,
        "4h" => 14400,
        "6h" => 21600,
        "8h" => 28800,
        "12h" => 43200,
        "1d" => 86400,
        "3d" => 259200,
        "1w" => 604800,
        "1M" => 2592000,
        _ => u64::MAX,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calc_kline_count() {
        assert_eq!(calc_kline_count("1s", 1502942428, 1502942437), 10);
        assert_eq!(calc_kline_count("1m", 1502942400, 1502942940), 10);
        assert_eq!(calc_kline_count("1d", 1502928000, 1503705600), 10);
    }

    #[test]
    fn test_calc_time_range_group() {
        assert_eq!(
            calc_time_range_group("1d", 1502928000, 1503705600, 5),
            vec![
                (1502928000000, 1503359999999),
                (1503360000000, 1503705600000)
            ]
        );

        assert_eq!(
            calc_time_range_group("1h", 1502928000, 1503705600, 48),
            vec![
                (1502928000000, 1503100799999),
                (1503100800000, 1503273599999),
                (1503273600000, 1503446399999),
                (1503446400000, 1503619199999),
                (1503619200000, 1503705600000)
            ]
        );
    }
}
