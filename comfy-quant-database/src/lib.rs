pub mod kline;
pub mod spot_pairs;
pub mod strategy_spot_position;
pub mod strategy_spot_stats;

pub static MIGRATOR: sqlx::migrate::Migrator = sqlx::migrate!("../migrations");

use bon::Builder;
use comfy_quant_base::{Exchange, Symbol};

#[derive(Debug, Builder)]
pub struct SpotStatsQuery<'a> {
    pub workflow_id: &'a str,
    pub node_id: i16,
    pub exchange: &'a Exchange,
    pub symbol: &'a Symbol,
}
