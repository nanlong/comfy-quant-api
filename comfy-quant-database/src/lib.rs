pub mod kline;
pub mod task;
pub mod utils;

pub static MIGRATOR: sqlx::migrate::Migrator = sqlx::migrate!("../migrations");
