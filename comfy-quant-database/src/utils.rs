use anyhow::{anyhow, Result};
use chrono::{DateTime, Datelike, Duration, Timelike, Utc};
use std::str::FromStr;

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

pub fn calc_interval_start(time: i64, unit: IntervalUnit, interval: u32) -> Result<i64> {
    let time = DateTime::from_timestamp(time, 0).ok_or_else(|| anyhow!("Invalid time"))?;

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

fn calc_interval_start_with_second(time: DateTime<Utc>, interval: u32) -> Result<i64> {
    let n = time.second() / interval;

    let timestamp = time
        .with_second(n * interval)
        .and_then(|t| t.with_nanosecond(0))
        .ok_or(anyhow!("Invalid time"))?
        .timestamp();

    Ok(timestamp)
}

fn calc_interval_start_with_minute(time: DateTime<Utc>, interval: u32) -> Result<i64> {
    let n = time.minute() / interval;

    let timestamp = time
        .with_minute(n * interval)
        .ok_or(anyhow!("Invalid time"))?
        .with_second(0)
        .ok_or(anyhow!("Invalid time"))?
        .with_nanosecond(0)
        .ok_or(anyhow!("Invalid time"))?
        .timestamp();

    Ok(timestamp)
}

fn calc_interval_start_with_hour(time: DateTime<Utc>, interval: u32) -> Result<i64> {
    let n = time.hour() / interval;

    let timestamp = time
        .with_hour(n * interval)
        .and_then(|t| t.with_minute(0))
        .and_then(|t| t.with_second(0))
        .and_then(|t| t.with_nanosecond(0))
        .ok_or_else(|| anyhow!("Invalid time"))?
        .timestamp();

    Ok(timestamp)
}

fn calc_interval_start_with_day(time: DateTime<Utc>, interval: u32) -> Result<i64> {
    let n = time.ordinal() / interval;

    let timestamp = time
        .with_ordinal(n * interval)
        .and_then(|t| t.with_hour(0))
        .and_then(|t| t.with_minute(0))
        .and_then(|t| t.with_second(0))
        .and_then(|t| t.with_nanosecond(0))
        .ok_or_else(|| anyhow!("Invalid time"))?
        .timestamp();

    Ok(timestamp)
}

fn calc_interval_start_with_week(time: DateTime<Utc>, interval: u32) -> Result<i64> {
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

fn calc_interval_start_with_month(time: DateTime<Utc>, interval: u32) -> Result<i64> {
    let n = time.month() / interval;

    let timestamp = time
        .with_month(n * interval)
        .and_then(|t| t.with_day(1))
        .and_then(|t| t.with_hour(0))
        .and_then(|t| t.with_minute(0))
        .and_then(|t| t.with_second(0))
        .and_then(|t| t.with_nanosecond(0))
        .ok_or_else(|| anyhow!("Invalid time"))?
        .timestamp();

    Ok(timestamp)
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
