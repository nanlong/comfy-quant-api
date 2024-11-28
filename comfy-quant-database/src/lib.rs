pub mod kline;
pub mod strategy_spot_position;
pub mod strategy_spot_stats;
pub mod utils;

pub static MIGRATOR: sqlx::migrate::Migrator = sqlx::migrate!("../migrations");

use bon::Builder;

#[derive(Debug, Builder)]
pub struct SpotStatsQuery<'a> {
    pub workflow_id: &'a str,
    pub node_id: i16,
    pub exchange: &'a str,
    pub symbol: &'a str,
}
