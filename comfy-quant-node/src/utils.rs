use anyhow::Result;
use chrono::{DateTime, NaiveDateTime, Utc};

pub(crate) fn add_utc_offset(datetime: &str) -> Result<DateTime<Utc>> {
    let naive = NaiveDateTime::parse_from_str(datetime, "%Y-%m-%d %H:%M:%S")?;
    let utc_time = DateTime::<Utc>::from_naive_utc_and_offset(naive, Utc);
    Ok(utc_time)
}

// 保留小数点位数，向下取整
pub(crate) fn floor_to(f: f64, decimals: u32) -> f64 {
    let scale = 10_u64.pow(decimals);
    (f * scale as f64).floor() / scale as f64
}

// 保留小数点位数，四舍五入
pub(crate) fn round_to(f: f64, decimals: u32) -> f64 {
    let scale = 10_u64.pow(decimals);
    (f * scale as f64).round() / scale as f64
}
