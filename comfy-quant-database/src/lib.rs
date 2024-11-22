pub mod kline;
pub mod strategy_spot_position;
pub mod strategy_spot_stats;
pub mod utils;

pub static MIGRATOR: sqlx::migrate::Migrator = sqlx::migrate!("../migrations");
