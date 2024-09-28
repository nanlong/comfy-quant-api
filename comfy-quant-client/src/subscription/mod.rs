mod binance_spot_ticker;
use anyhow::Result;
use sqlx::{Pool, Postgres};
use std::future::Future;

pub use binance_spot_ticker::{BinanceSpotTicker, TickerWrapper};

pub trait Subscription {
    fn execute(&self, pool: &Pool<Postgres>) -> impl Future<Output = Result<()>> + Send;
}
