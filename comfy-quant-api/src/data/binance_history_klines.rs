use anyhow::Result;
use async_stream::try_stream;
use binance::model::{KlineSummaries, KlineSummary};
use comfy_quant_client::BinanceClient;
use futures::Stream;

#[allow(unused)]
pub enum Market {
    Spot,
    Futures,
}

pub struct BinanceHistoryKlines {
    client: BinanceClient,
}

#[allow(unused)]
impl BinanceHistoryKlines {
    pub fn new() -> Self {
        let client = BinanceClient::builder().build();
        BinanceHistoryKlines { client }
    }

    // 获取K线流
    pub fn klines_stream<'a>(
        &'a self,
        market: Market,    // 市场
        symbol: &'a str,   // 交易对
        interval: &'a str, // 时间间隔
        start_time: u64,   // 开始时间
        end_time: u64,     // 结束时间
    ) -> impl Stream<Item = Result<KlineSummary>> + 'a {
        try_stream! {
            let limit = 1000;
            let groups = calc_time_range_group(interval, start_time, end_time, limit);

            for (start_time, end_time) in groups {
                let KlineSummaries::AllKlineSummaries(klines) = match market {
                    Market::Spot => self.client.spot().get_klines(
                        symbol,
                        interval,
                        limit as u16,
                        start_time,
                        end_time,
                    )?,
                    Market::Futures => self.client.futures().get_klines(
                        symbol,
                        interval,
                        limit as u16,
                        start_time,
                        end_time,
                    )?,
                };

                for kline in klines {
                    yield kline;
                }
            }
        }
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
