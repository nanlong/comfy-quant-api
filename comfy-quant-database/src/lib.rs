pub mod kline;
pub mod strategy_position;
pub mod utils;

pub static MIGRATOR: sqlx::migrate::Migrator = sqlx::migrate!("../migrations");
