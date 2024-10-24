use crate::{
    node_core::{Executable, Port, PortAccessor, Setupable, Slot},
    node_io::{SpotPairInfo, Tick, TickStream},
    utils::add_utc_offset,
    workflow::{self, WorkflowContext},
};
use anyhow::{anyhow, Result};
use bon::Builder;
use chrono::{DateTime, Utc};
use comfy_quant_database::kline;
use comfy_quant_task::{
    task_core::{status::TaskStatus, traits::Executable as _},
    tasks::binance_klines::BinanceKlinesTask,
};
use futures::StreamExt;
use std::sync::Arc;
use tokio::sync::Barrier;

const EXCHANGE: &str = "binance";
const MARKET: &str = "spot";
const INTERVAL: &str = "1s";

#[derive(Builder, Debug, Clone)]
#[builder(on(String, into))]
pub(crate) struct Params {
    pub(crate) base_asset: String,
    pub(crate) quote_asset: String,
    pub(crate) start_datetime: DateTime<Utc>,
    pub(crate) end_datetime: DateTime<Utc>,
}

/// 回测行情数据
/// outputs:
///      0: SpotPairInfo
///      1: TickStream
#[derive(Debug)]
pub(crate) struct BacktestSpotTicker {
    pub(crate) params: Params,
    pub(crate) port: Port,
    context: Option<WorkflowContext>,
    barrier: Arc<Barrier>,
    shutdown_tx: flume::Sender<()>,
    shutdown_rx: flume::Receiver<()>,
}

impl BacktestSpotTicker {
    pub(crate) fn try_new(params: Params) -> Result<Self> {
        let mut port = Port::new();

        let pair_info = SpotPairInfo::builder()
            .base_asset(&params.base_asset)
            .quote_asset(&params.quote_asset)
            .build();
        let tick_stream = TickStream::new();

        let pair_info_slot = Slot::<SpotPairInfo>::new(pair_info);
        let tick_stream_slot = Slot::<TickStream>::new(tick_stream);

        let barrier = Arc::new(Barrier::new(2));
        let (shutdown_tx, shutdown_rx) = flume::bounded(1);

        port.add_output(0, pair_info_slot)?;
        port.add_output(1, tick_stream_slot)?;

        Ok(BacktestSpotTicker {
            params,
            port,
            context: None,
            barrier,
            shutdown_tx,
            shutdown_rx,
        })
    }

    async fn output1(&self) -> Result<()> {
        let slot1 = self.port.get_output::<TickStream>(1)?;
        let symbol =
            format!("{}{}", self.params.base_asset, self.params.quote_asset).to_uppercase();
        let symbol_cloned = symbol.clone();
        let start_timestamp = self.params.start_datetime.timestamp();
        let end_timestamp = self.params.end_datetime.timestamp();

        let context = self
            .context
            .as_ref()
            .ok_or_else(|| anyhow!("context not setup"))?;
        let task1_db = context.db()?;
        let task2_db = context.db()?;
        let task1_barrier = Arc::clone(&self.barrier);
        let task2_barrier = Arc::clone(&self.barrier);
        let task1_shutdown_rx = self.shutdown_rx.clone();
        let task2_shutdown_rx = self.shutdown_rx.clone();

        tokio::spawn(async move {
            tokio::select! {
                _ = async {
                    // 等待数据同步完成，如果出错，重试3次
                    'retry: for i in 0..3 {
                        let task = BinanceKlinesTask::builder()
                            .db(Arc::clone(&task1_db))
                            .market(MARKET)
                            .symbol(&symbol)
                            .interval(INTERVAL)
                            .start_timestamp(start_timestamp)
                            .end_timestamp(end_timestamp)
                            .build();

                        let receiver = task.execute().await?;

                        while let Ok(status) = receiver.recv_async().await {
                            match status {
                                TaskStatus::Running => {
                                    tracing::info!("Binance klines task running");
                                }
                                TaskStatus::Finished => {
                                    tracing::info!("Binance klines task finished");
                                    break 'retry;
                                }
                                TaskStatus::Failed(err) => {
                                    tracing::error!("{} Binance klines task failed: {}", i + 1, err);
                                    continue 'retry;
                                }
                                _ => {}
                            }
                        }
                    }

                    task1_barrier.wait().await;

                    Ok::<(), anyhow::Error>(())
                } => {}
                _ = task1_shutdown_rx.recv_async() => {
                    tracing::info!("BacktestSpotTicker task1 shutdown");
                }
            }
        });

        tokio::spawn(async move {
            tokio::select! {
                _ = async {
                    task2_barrier.wait().await;

                    let mut klines_stream = kline::time_range_klines_stream(
                        &task2_db,
                        EXCHANGE,
                        MARKET,
                        &symbol_cloned,
                        INTERVAL,
                        start_timestamp * 1000,
                        end_timestamp * 1000,
                    );

                    while let Some(Ok(kline)) = klines_stream.next().await {
                        let tick = Tick::builder()
                            .timestamp(kline.open_time / 1000)
                            .price(kline.close_price.to_string().parse::<f64>()?)
                            .build();

                        tracing::info!("BacktestSpotTicker send tick: {:?}", tick);

                        slot1.send(tick).await?;
                    }

                    Ok::<(), anyhow::Error>(())
                } => {}
                _ = task2_shutdown_rx.recv_async() => {
                    tracing::info!("BacktestSpotTicker task2 shutdown");
                }
            }
        });

        Ok(())
    }
}

impl Setupable for BacktestSpotTicker {
    fn setup_context(&mut self, context: WorkflowContext) {
        self.context = Some(context);
    }
}

impl PortAccessor for BacktestSpotTicker {
    fn get_port(&self) -> Result<&Port> {
        Ok(&self.port)
    }

    fn get_port_mut(&mut self) -> Result<&mut Port> {
        Ok(&mut self.port)
    }
}

impl Executable for BacktestSpotTicker {
    async fn execute(&mut self) -> Result<()> {
        self.context
            .as_ref()
            .ok_or_else(|| anyhow!("context not setup"))?
            .wait()
            .await?;
        self.output1().await?;
        Ok(())
    }
}

impl Drop for BacktestSpotTicker {
    fn drop(&mut self) {
        let _ = self.shutdown_tx.send(());
    }
}

impl TryFrom<workflow::Node> for BacktestSpotTicker {
    type Error = anyhow::Error;

    fn try_from(node: workflow::Node) -> Result<Self> {
        if node.properties.prop_type != "data.BacktestSpotTicker" {
            anyhow::bail!(
                "Try from workflow::Node to BacktestSpotTicker failed: Invalid prop_type"
            );
        }

        let [base_asset, quote_asset, start_datetime, end_datetime] =
            node.properties.params.as_slice()
        else {
            anyhow::bail!("Try from workflow::Node to BacktestSpotTicker failed: Invalid params");
        };

        let base_asset = base_asset.as_str().ok_or(anyhow::anyhow!(
            "Try from workflow::Node to BacktestSpotTicker failed: Invalid base_asset"
        ))?;

        let quote_asset = quote_asset.as_str().ok_or(anyhow::anyhow!(
            "Try from workflow::Node to BacktestSpotTicker failed: Invalid quote_asset"
        ))?;

        let start_datetime = start_datetime.as_str().ok_or(anyhow::anyhow!(
            "Try from workflow::Node to BacktestSpotTicker failed: Invalid start_datetime"
        ))?;

        let end_datetime = end_datetime.as_str().ok_or(anyhow::anyhow!(
            "Try from workflow::Node to BacktestSpotTicker failed: Invalid end_datetime"
        ))?;

        let start_datetime = add_utc_offset(start_datetime)?;
        let end_datetime = add_utc_offset(end_datetime)?;

        let params = Params::builder()
            .base_asset(base_asset)
            .quote_asset(quote_asset)
            .start_datetime(start_datetime)
            .end_datetime(end_datetime)
            .build();

        BacktestSpotTicker::try_new(params)
    }
}
