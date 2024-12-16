use std::fmt;

use strum_macros::{AsRefStr, EnumIter};

#[derive(Debug, Default, Clone, PartialEq, Eq, AsRefStr, EnumIter)]
pub enum KlineInterval {
    #[strum(serialize = "1s")]
    OneSecond,
    #[strum(serialize = "1m")]
    OneMinute,
    #[strum(serialize = "3m")]
    ThreeMinutes,
    #[strum(serialize = "5m")]
    FiveMinutes,
    #[strum(serialize = "15m")]
    FifteenMinutes,
    #[strum(serialize = "30m")]
    ThirtyMinutes,
    #[strum(serialize = "1h")]
    OneHour,
    #[strum(serialize = "2h")]
    TwoHours,
    #[strum(serialize = "4h")]
    FourHours,
    #[strum(serialize = "6h")]
    SixHours,
    #[strum(serialize = "8h")]
    EightHours,
    #[strum(serialize = "12h")]
    TwelveHours,
    #[strum(serialize = "1d")]
    OneDay,
    #[strum(serialize = "3d")]
    ThreeDays,
    #[strum(serialize = "1w")]
    OneWeek,
    #[strum(serialize = "1M")]
    OneMonth,
    #[default]
    Unknow,
}

impl From<&str> for KlineInterval {
    fn from(value: &str) -> Self {
        match value {
            "1s" => KlineInterval::OneSecond,
            "1m" => KlineInterval::OneMinute,
            "3m" => KlineInterval::ThreeMinutes,
            "5m" => KlineInterval::FiveMinutes,
            "15m" => KlineInterval::FifteenMinutes,
            "30m" => KlineInterval::ThirtyMinutes,
            "1h" => KlineInterval::OneHour,
            "2h" => KlineInterval::TwoHours,
            "4h" => KlineInterval::FourHours,
            "6h" => KlineInterval::SixHours,
            "8h" => KlineInterval::EightHours,
            "12h" => KlineInterval::TwelveHours,
            "1d" => KlineInterval::OneDay,
            "3d" => KlineInterval::ThreeDays,
            "1w" => KlineInterval::OneWeek,
            "1M" => KlineInterval::OneMonth,
            _ => KlineInterval::Unknow,
        }
    }
}

impl From<String> for KlineInterval {
    fn from(value: String) -> Self {
        value.as_str().into()
    }
}

impl From<&KlineInterval> for String {
    fn from(value: &KlineInterval) -> Self {
        value.as_ref().to_string()
    }
}

impl From<KlineInterval> for String {
    fn from(value: KlineInterval) -> Self {
        value.as_ref().to_string()
    }
}

impl fmt::Display for KlineInterval {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_ref())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_kline_interval() {
        let interval = KlineInterval::OneMinute;
        assert_eq!(interval.as_ref(), "1m");

        let interval2: KlineInterval = "1m".into();
        assert_eq!(interval2, KlineInterval::OneMinute);

        let interval3: KlineInterval = "1s".into();
        assert_eq!(interval3, KlineInterval::OneSecond);
    }
}
