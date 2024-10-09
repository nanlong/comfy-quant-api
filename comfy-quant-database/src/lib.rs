pub mod kline;
pub mod utils;

pub static MIGRATOR: sqlx::migrate::Migrator = sqlx::migrate!("../migrations");
