use chrono::{DateTime, Utc};

#[derive(Debug, Clone)]
pub struct BacktestConfig {
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_backtest_config_new() -> anyhow::Result<()> {
        let start_time = "2024-01-01T00:00:00Z".parse::<DateTime<Utc>>()?;
        let end_time = "2024-01-02T23:59:59Z".parse::<DateTime<Utc>>()?;
        let backtest = BacktestConfig {
            start_time,
            end_time,
        };

        assert_eq!(backtest.start_time, start_time);
        assert_eq!(backtest.end_time, end_time);

        Ok(())
    }
}
