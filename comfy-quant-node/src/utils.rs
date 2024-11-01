use anyhow::Result;
use chrono::{DateTime, NaiveDateTime, Utc};

const ALPHABET: &[char] = &[
    '1', '2', '3', '4', '5', '6', '7', '8', '9', 'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'J', 'K',
    'L', 'M', 'N', 'P', 'Q', 'R', 'S', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z', 'a', 'b', 'c', 'd', 'e',
    'f', 'g', 'h', 'i', 'j', 'k', 'm', 'n', 'o', 'p', 'q', 'r', 's', 't', 'u', 'v', 'w', 'x', 'y',
    'z',
];

pub(crate) fn add_utc_offset(datetime: &str) -> Result<DateTime<Utc>> {
    let naive = NaiveDateTime::parse_from_str(datetime, "%Y-%m-%d %H:%M:%S")?;
    let utc_time = DateTime::<Utc>::from_naive_utc_and_offset(naive, Utc);
    Ok(utc_time)
}

// 生成21位唯一ID
pub(crate) fn generate_workflow_id() -> String {
    nanoid::nanoid!(21, &ALPHABET)
}
