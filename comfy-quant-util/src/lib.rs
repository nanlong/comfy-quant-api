use anyhow::{anyhow, Result};
use chrono::{DateTime, Datelike, Duration, NaiveDateTime, Timelike, Utc};
use std::str::FromStr;

pub const ALPHABET: &[char] = &[
    '1', '2', '3', '4', '5', '6', '7', '8', '9', 'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'J', 'K',
    'L', 'M', 'N', 'P', 'Q', 'R', 'S', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z', 'a', 'b', 'c', 'd', 'e',
    'f', 'g', 'h', 'i', 'j', 'k', 'm', 'n', 'o', 'p', 'q', 'r', 's', 't', 'u', 'v', 'w', 'x', 'y',
    'z',
];

pub fn add_utc_offset(datetime: &str) -> Result<DateTime<Utc>> {
    let naive = NaiveDateTime::parse_from_str(datetime, "%Y-%m-%d %H:%M:%S")?;
    let utc_time = DateTime::<Utc>::from_naive_utc_and_offset(naive, Utc);
    Ok(utc_time)
}

// 生成21位唯一ID
pub fn generate_workflow_id() -> String {
    nanoid::nanoid!(21, &ALPHABET)
}

// 将秒转换为UTC时间
pub fn secs_to_datetime(secs: impl Into<i64>) -> Result<DateTime<Utc>> {
    let datetime = DateTime::<Utc>::from_timestamp(secs.into(), 0)
        .ok_or_else(|| anyhow::anyhow!("secs is invalid"))?;

    Ok(datetime)
}

// 将毫秒转换为UTC时间
#[allow(unused)]
pub fn millis_to_datetime(millis: impl Into<i64>) -> Result<DateTime<Utc>> {
    let datetime = DateTime::<Utc>::from_timestamp_millis(millis.into())
        .ok_or_else(|| anyhow::anyhow!("millis is invalid"))?;

    Ok(datetime)
}

// 计算时间间隔的开始时间
pub fn calc_interval_start(time: i64, unit: IntervalUnit, interval: u32) -> Result<i64> {
    let time = secs_to_datetime(time).ok();

    let start_time = match unit {
        IntervalUnit::Second => calc_interval_start_with_second(time, interval),
        IntervalUnit::Minute => calc_interval_start_with_minute(time, interval),
        IntervalUnit::Hour => calc_interval_start_with_hour(time, interval),
        IntervalUnit::Day => calc_interval_start_with_day(time, interval),
        IntervalUnit::Week => calc_interval_start_with_week(time, interval),
        IntervalUnit::Month => calc_interval_start_with_month(time, interval),
    }?;

    Ok(start_time)
}

fn calc_interval_start_with_second(time: Option<DateTime<Utc>>, interval: u32) -> Result<i64> {
    let timestamp = time
        .and_then(|t| t.with_second(t.second() / interval * interval))
        .and_then(|t| t.with_nanosecond(0))
        .ok_or(anyhow!("Invalid time"))?
        .timestamp();

    Ok(timestamp)
}

fn calc_interval_start_with_minute(time: Option<DateTime<Utc>>, interval: u32) -> Result<i64> {
    let timestamp = time
        .and_then(|t| t.with_minute(t.minute() / interval * interval))
        .and_then(|t| t.with_second(0))
        .and_then(|t| t.with_nanosecond(0))
        .ok_or(anyhow!("Invalid time"))?
        .timestamp();

    Ok(timestamp)
}

fn calc_interval_start_with_hour(time: Option<DateTime<Utc>>, interval: u32) -> Result<i64> {
    let timestamp = time
        .and_then(|t| t.with_hour(t.hour() / interval * interval))
        .and_then(|t| t.with_minute(0))
        .and_then(|t| t.with_second(0))
        .and_then(|t| t.with_nanosecond(0))
        .ok_or_else(|| anyhow!("Invalid time"))?
        .timestamp();

    Ok(timestamp)
}

fn calc_interval_start_with_day(time: Option<DateTime<Utc>>, interval: u32) -> Result<i64> {
    let timestamp = time
        .and_then(|t| t.with_ordinal(t.ordinal() / interval * interval))
        .and_then(|t| t.with_hour(0))
        .and_then(|t| t.with_minute(0))
        .and_then(|t| t.with_second(0))
        .and_then(|t| t.with_nanosecond(0))
        .ok_or_else(|| anyhow!("Invalid time"))?
        .timestamp();

    Ok(timestamp)
}

fn calc_interval_start_with_week(time: Option<DateTime<Utc>>, interval: u32) -> Result<i64> {
    let time = time.ok_or_else(|| anyhow!("Invalid time"))?;

    let monday = time
        - Duration::days(time.weekday().num_days_from_monday() as i64)
        - Duration::days(7 * (interval - 1) as i64);

    let timestamp = monday
        .with_hour(0)
        .and_then(|t| t.with_minute(0))
        .and_then(|t| t.with_second(0))
        .and_then(|t| t.with_nanosecond(0))
        .ok_or_else(|| anyhow!("Invalid time"))?
        .timestamp();

    Ok(timestamp)
}

fn calc_interval_start_with_month(time: Option<DateTime<Utc>>, interval: u32) -> Result<i64> {
    let timestamp = time
        .and_then(|t| t.with_month(t.month() / interval * interval))
        .and_then(|t| t.with_day(1))
        .and_then(|t| t.with_hour(0))
        .and_then(|t| t.with_minute(0))
        .and_then(|t| t.with_second(0))
        .and_then(|t| t.with_nanosecond(0))
        .ok_or_else(|| anyhow!("Invalid time"))?
        .timestamp();

    Ok(timestamp)
}

pub enum IntervalUnit {
    Second,
    Minute,
    Hour,
    Day,
    Week,
    Month,
}

impl FromStr for IntervalUnit {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "1s" => Ok(IntervalUnit::Second),
            "1m" | "3m" | "5m" | "15m" | "30m" => Ok(IntervalUnit::Minute),
            "1h" | "2h" | "4h" | "6h" | "8h" | "12h" => Ok(IntervalUnit::Hour),
            "1d" | "3d" => Ok(IntervalUnit::Day),
            "1w" => Ok(IntervalUnit::Week),
            "1M" => Ok(IntervalUnit::Month),
            _ => Err(anyhow!("Invalid interval unit: {}", s)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // 2024-04-26 13:01:22
    static TEST_TIME: i64 = 1714107682;

    #[test]
    fn test_get_1m_interval_start() -> Result<()> {
        let start = calc_interval_start(TEST_TIME, IntervalUnit::Minute, 1)?;
        assert_eq!(start, 1714107660);

        Ok(())
    }

    #[test]
    fn test_get_1h_interval_start() -> Result<()> {
        let start = calc_interval_start(TEST_TIME, IntervalUnit::Hour, 1)?;
        assert_eq!(start, 1714107600);

        Ok(())
    }

    #[test]
    fn test_get_1d_interval_start() -> Result<()> {
        let start = calc_interval_start(TEST_TIME, IntervalUnit::Day, 1)?;
        assert_eq!(start, 1714089600);

        Ok(())
    }

    #[test]
    fn test_get_1w_interval_start() -> Result<()> {
        let start = calc_interval_start(TEST_TIME, IntervalUnit::Week, 1)?;
        assert_eq!(start, 1713744000);

        Ok(())
    }

    #[test]
    fn test_get_1month_interval_start() -> Result<()> {
        let start = calc_interval_start(TEST_TIME, IntervalUnit::Month, 1)?;
        assert_eq!(start, 1711929600);

        Ok(())
    }
}
