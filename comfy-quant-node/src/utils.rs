use anyhow::Result;
use chrono::{DateTime, NaiveDateTime, Utc};

pub fn add_utc_offset(datetime: &str) -> Result<DateTime<Utc>> {
    let naive = NaiveDateTime::parse_from_str(datetime, "%Y-%m-%d %H:%M:%S")?;
    let utc_time = DateTime::<Utc>::from_naive_utc_and_offset(naive, Utc);
    Ok(utc_time)
}
