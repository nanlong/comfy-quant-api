// 计算时间范围内的K线数量
pub fn calc_time_range_kline_count(
    interval: &str,
    start_timestamp: i64,
    end_timestamp: i64,
) -> usize {
    let seconds = interval_to_seconds(interval);
    ((end_timestamp - start_timestamp) / seconds + 1) as usize
}

// 计算时间范围内的K线分组
pub fn calc_time_range_group(
    interval: &str,
    start_timestamp: i64,
    end_timestamp: i64,
    limit: u16,
) -> Vec<(i64, i64)> {
    let mut result = Vec::new();
    let mut start_time = start_timestamp;
    let interval_seconds = interval_to_seconds(interval);
    let kline_count = calc_time_range_kline_count(interval, start_timestamp, end_timestamp) as i64;

    for _i in 0..=(kline_count / limit as i64) {
        let end_time = start_time + interval_seconds * limit as i64;

        if end_time > end_timestamp {
            result.push((start_time * 1000, end_timestamp * 1000));
            break;
        } else {
            result.push((start_time * 1000, end_time * 1000 - 1));
            start_time = end_time;
        }
    }

    result
}

// 将时间间隔转换为秒
pub fn interval_to_seconds(interval: &str) -> i64 {
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
        _ => i64::MAX,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calc_time_range_kline_count() {
        assert_eq!(
            calc_time_range_kline_count("1s", 1502942428, 1502942437),
            10
        );
        assert_eq!(
            calc_time_range_kline_count("1m", 1502942400, 1502942940),
            10
        );
        assert_eq!(
            calc_time_range_kline_count("1d", 1502928000, 1503705600),
            10
        );
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
