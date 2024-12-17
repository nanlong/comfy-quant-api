use super::utils::calc_time_range_group;
use crate::exchange::binance::BinanceClient;
use anyhow::Result;
use async_stream::stream;
use binance::{
    config::Config,
    model::{KlineSummaries, KlineSummary},
};
use bon::bon;
use comfy_quant_base::{KlineInterval, Market, Symbol};
use futures::stream::BoxStream;
use std::sync::Arc;
use tokio_util::sync::CancellationToken;

const KLINE_LIMIT: u16 = 1000;

#[derive(Debug)]
pub struct BinanceKline {
    client: Arc<BinanceClient>,
    token: CancellationToken,
}

#[bon]
impl BinanceKline {
    #[builder]
    pub fn new(config: Option<Config>) -> Self {
        // 访问公共接口，不需要api_key和secret_key
        let client = Arc::new(BinanceClient::builder().maybe_config(config).build());
        let token = CancellationToken::new();

        BinanceKline { client, token }
    }

    // 获取K线流
    pub fn klines_stream(
        &self,
        market: &Market,          // 市场
        symbol: &Symbol,          // 交易对
        interval: &KlineInterval, // 时间间隔
        start_time: i64,          // 开始时间
        end_time: i64,            // 结束时间
    ) -> BoxStream<Result<KlineSummary>> {
        let client = Arc::clone(&self.client);
        let (tx, rx) = flume::bounded(1);
        let (error_tx, error_rx) = flume::bounded(1);
        let semaphore = Arc::new(async_lock::Semaphore::new(1));
        let time_range_groups =
            calc_time_range_group(interval.as_ref(), start_time, end_time, KLINE_LIMIT);
        let cloned_token1 = self.token.clone();
        let cloned_token2 = self.token.clone();

        // 使用 tokio::spawn 会有问题，所以使用 tokio::task::spawn_blocking
        // 报错信息: Cannot drop a runtime in a context where blocking is not allowed. This happens when a runtime is dropped from within an asynchronous context.
        // 原因: reqwest 的 runtime 在异步上下文中被释放了
        tokio::task::spawn_blocking({
            let market = market.clone();
            let symbol = symbol.clone();
            let interval = interval.clone();

            move || {
                let result = (move || {
                    for (start_time, end_time) in time_range_groups {
                        if cloned_token1.is_cancelled() {
                            return Ok(());
                        }

                        let KlineSummaries::AllKlineSummaries(klines) = match market {
                            Market::Spot => client.spot().get_klines(
                                &symbol,
                                &interval,
                                KLINE_LIMIT,
                                start_time as u64,
                                end_time as u64,
                            )?,
                            Market::Usdm | Market::Coinm | Market::Vanilla => {
                                client.futures().get_klines(
                                    &symbol,
                                    &interval,
                                    KLINE_LIMIT,
                                    start_time as u64,
                                    end_time as u64,
                                )?
                            }
                        };

                        for kline in klines {
                            if cloned_token1.is_cancelled() {
                                return Ok(());
                            }

                            let guard = semaphore.acquire_arc_blocking();
                            tx.send((kline, guard))?;
                        }
                    }

                    Ok(())
                })();

                if let Err(e) = result {
                    error_tx.send(e)?;
                }

                Ok::<(), anyhow::Error>(())
            }
        });

        let kline_stream = stream! {
            loop {
                if cloned_token2.is_cancelled() {
                    break;
                }

                tokio::select! {
                    Ok((kline, _guard)) = rx.recv_async() => {
                        yield Ok(kline);
                    }
                    Ok(err) = error_rx.recv_async() => {
                        yield Err(err);
                        break;
                    }
                    else => break,
                }
            }
        };

        Box::pin(kline_stream)
    }
}

impl Default for BinanceKline {
    fn default() -> Self {
        BinanceKline::builder().build()
    }
}

impl Drop for BinanceKline {
    fn drop(&mut self) {
        self.token.cancel();
    }
}
