use super::Subscription;
use anyhow::Result;
use binance::{
    model::DayTickerEvent,
    websockets::{WebSockets, WebsocketEvent},
};
use comfy_quant_data::{
    kline::{self, Kline},
    utils::{calc_interval_start, IntervalUnit},
};
use rust_decimal::Decimal;
use sqlx::PgPool;
use std::{
    ops::Deref,
    str::FromStr,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread::sleep,
    time::Duration,
};
use tokio::sync::broadcast;
use tokio_stream::{wrappers::BroadcastStream, StreamExt};

pub struct BinanceSpotTicker {
    sender: Arc<broadcast::Sender<TickerWrapper>>,
}

impl BinanceSpotTicker {
    pub fn new() -> Self {
        let (sender, _receiver) = broadcast::channel(1024);

        Self {
            sender: Arc::new(sender),
        }
    }

    pub async fn all_trades(&self) -> Result<()> {
        let sender = self.sender.clone();

        let _handle = tokio::spawn(async move {
            let keep_running = AtomicBool::new(true); // Used to control the event loop
            let agg_trade = String::from("!ticker@arr");
            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

            let mut web_socket = WebSockets::new(|event: WebsocketEvent| {
                if let WebsocketEvent::DayTickerAll(ticker_events) = event {
                    for tick_event in ticker_events {
                        sender.send(TickerWrapper(tick_event)).map_err(|e| {
                            binance::errors::Error::from(binance::errors::ErrorKind::Msg(
                                e.to_string(),
                            ))
                        })?;
                    }
                }

                Ok(())
            });

            'reconnect: loop {
                eprintln!("reconnect");

                if let Err(_e) = web_socket.connect(&agg_trade) {
                    sleep(Duration::from_secs(1));
                    continue 'reconnect;
                };

                while keep_running.load(Ordering::Relaxed) {
                    if let Err(_e) = web_socket.event_loop(&keep_running) {
                        sleep(Duration::from_secs(1));
                        continue 'reconnect;
                    }
                }

                if let Err(_e) = web_socket.disconnect() {
                    sleep(Duration::from_secs(1));
                    continue 'reconnect;
                };
            }

            #[allow(unreachable_code)]
            Ok::<(), anyhow::Error>(())
        });

        Ok(())
    }
}

impl Default for BinanceSpotTicker {
    fn default() -> Self {
        Self::new()
    }
}

impl Subscription for BinanceSpotTicker {
    async fn execute(&self, pool: &PgPool) -> Result<()> {
        self.all_trades().await?;
        let tx = self.sender.clone();
        let pool = pool.clone();

        let mut stream = BroadcastStream::new(tx.subscribe());

        while let Some(Ok(ticker_wrapper)) = stream.next().await {
            let intervals = vec![
                "1s", "1m", "3m", "5m", "15m", "30m", "1h", "2h", "4h", "6h", "8h", "12h", "1d",
                "3d", "1w", "1M",
            ];

            for interval in intervals {
                let ticker_wrapper = ticker_wrapper.clone();
                let pool = pool.clone();

                tokio::spawn(async move {
                    let item = ticker_wrapper.try_into_kline(&pool, interval).await?;
                    kline::insert_or_update(&pool, &item).await?;
                    Ok::<(), anyhow::Error>(())
                });
            }
        }

        Ok(())
    }
}

static EXCHANGE: &str = "binance";

#[derive(Debug, Clone)]
pub struct TickerWrapper(pub DayTickerEvent);

impl Deref for TickerWrapper {
    type Target = DayTickerEvent;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl TickerWrapper {
    pub async fn try_into_kline(&self, pool: &PgPool, interval: &str) -> Result<Kline> {
        let symbol = self.symbol.clone();
        let open_time = self.open_time / 1000;
        let current_close = self.current_close.clone();
        let current_close_qty = self.current_close_qty.clone();
        let price = Decimal::from_str(&current_close)?;
        let volume = Decimal::from_str(&current_close_qty)?;

        let interval_unit = interval.parse::<IntervalUnit>()?;

        let interval_count = interval
            .chars()
            .take(interval.len() - 1)
            .collect::<String>()
            .parse::<u32>()?;

        let start_time = calc_interval_start(open_time as i64, interval_unit, interval_count)?;

        let kline =
            match kline::get_kline(pool, EXCHANGE, &symbol, interval, start_time as i64).await? {
                Some(kline) => Kline {
                    high_price: price.max(kline.high_price),
                    low_price: price.min(kline.low_price),
                    close_price: price,
                    volume: kline.volume + volume,
                    ..kline
                },
                None => Kline {
                    exchange: EXCHANGE.to_string(),
                    symbol,
                    interval: interval.to_string(),
                    open_time: start_time as i64,
                    open_price: price,
                    high_price: price,
                    low_price: price,
                    close_price: price,
                    volume,
                    ..Default::default()
                },
            };

        Ok(kline)
    }
}
