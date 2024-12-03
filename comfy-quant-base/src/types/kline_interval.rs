use std::fmt;

use strum_macros::{AsRefStr, EnumIter, EnumString};

#[derive(Debug, Clone, PartialEq, Eq, EnumString, AsRefStr, EnumIter)]
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

        let interval2 = "1m".parse::<KlineInterval>().unwrap();
        assert_eq!(interval2, KlineInterval::OneMinute);

        let interval3 = "1s".parse::<KlineInterval>().unwrap();
        assert_eq!(interval3, KlineInterval::OneSecond);

        let err = "1x".parse::<KlineInterval>().unwrap_err();
        assert_eq!(err.to_string(), "Matching variant not found");
    }
}
