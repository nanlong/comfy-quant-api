use anyhow::Result;
use chrono::{DateTime, NaiveDateTime, Utc};

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
