pub mod kline;

pub static MIGRATOR: sqlx::migrate::Migrator = sqlx::migrate!("../migrations");
