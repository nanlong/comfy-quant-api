use crate::{
    node_core::{NodeExecutable, NodeInfo, NodePort, Port, Slot, Tick},
    node_io::{SpotPairInfo, TickStream},
    utils::add_utc_offset,
    workflow::Node,
};
use anyhow::Result;
use bon::Builder;
use chrono::{DateTime, Utc};
use comfy_quant_database::kline;
use comfy_quant_task::{
    task_core::{status::TaskStatus, traits::Executable as _},
    tasks::binance_klines::BinanceKlinesTask,
};
use futures::StreamExt;
use std::sync::Arc;

const EXCHANGE: &str = "binance";
const MARKET: &str = "spot";
const INTERVAL: &str = "1s";

#[derive(Builder, Debug, Clone)]
#[builder(on(String, into))]
pub(crate) struct Params {
    base_asset: String,
    quote_asset: String,
    start_datetime: DateTime<Utc>,
    end_datetime: DateTime<Utc>,
}

/// 回测行情数据
/// outputs:
///      0: SpotPairInfo
///      1: TickStream
#[derive(Debug)]
#[allow(unused)]
pub(crate) struct BacktestSpotTicker {
    node: Node,
    params: Params,
    port: Port,
}

impl BacktestSpotTicker {
    pub(crate) fn try_new(node: Node, params: Params) -> Result<Self> {
        let mut port = Port::new();

        let pair_info = SpotPairInfo::builder()
            .base_asset(&params.base_asset)
            .quote_asset(&params.quote_asset)
            .build();
        let tick_stream = TickStream::new();

        let pair_info_slot = Slot::<SpotPairInfo>::new(pair_info);
        let tick_stream_slot = Slot::<TickStream>::new(tick_stream);

        port.set_output(0, pair_info_slot)?;
        port.set_output(1, tick_stream_slot)?;

        Ok(BacktestSpotTicker { node, params, port })
    }

    async fn output1(&self) -> Result<()> {
        let port = self.port();
        let slot1 = port.output::<TickStream>(1)?;
        let symbol =
            format!("{}{}", self.params.base_asset, self.params.quote_asset).to_uppercase();
        let start_timestamp = self.params.start_datetime.timestamp();
        let end_timestamp = self.params.end_datetime.timestamp();
        let db = self.node().context()?.cloned_db();

        // 等待数据同步完成，如果出错，重试3次
        'retry: for i in 0..3 {
            let task = BinanceKlinesTask::builder()
                .db(Arc::clone(&db))
                .market(MARKET)
                .symbol(&symbol)
                .interval(INTERVAL)
                .start_timestamp(start_timestamp)
                .end_timestamp(end_timestamp)
                .build();

            let mut task_result = task.execute().await?;

            tracing::info!("Binance klines task start");

            while let Some(Ok(status)) = task_result.next().await {
                match status {
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

        let mut klines_stream = kline::time_range_klines_stream(
            &db,
            EXCHANGE,
            MARKET,
            &symbol,
            INTERVAL,
            start_timestamp * 1000,
            end_timestamp * 1000,
        );

        while let Some(Ok(kline)) = klines_stream.next().await {
            let tick = Tick::builder()
                .timestamp(kline.open_time / 1000)
                .symbol(symbol.clone())
                .price(kline.close_price)
                .build();

            slot1.send(tick).await?;
        }

        Ok(())
    }
}

impl NodePort for BacktestSpotTicker {
    fn port(&self) -> &Port {
        &self.port
    }

    fn port_mut(&mut self) -> &mut Port {
        &mut self.port
    }
}

impl NodeInfo for BacktestSpotTicker {
    fn node(&self) -> &Node {
        &self.node
    }

    fn node_id(&self) -> i16 {
        self.node.node_id()
    }

    fn node_name(&self) -> &str {
        self.node.node_name()
    }
}

impl NodeExecutable for BacktestSpotTicker {
    async fn execute(&mut self) -> Result<()> {
        // 同步等待其他节点
        self.node().context()?.wait().await;

        self.output1().await?;
        Ok(())
    }
}

impl TryFrom<Node> for BacktestSpotTicker {
    type Error = anyhow::Error;

    fn try_from(node: Node) -> Result<Self> {
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

        BacktestSpotTicker::try_new(node, params)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_try_from_node_to_backtest_spot_ticker() -> anyhow::Result<()> {
        let json_str = r#"{"id":1,"type":"数据/币安现货行情","pos":[199,74],"size":{"0":210,"1":310},"flags":{},"order":0,"mode":0,"inputs":[],"properties":{"type":"data.BacktestSpotTicker","params":["BTC","USDT","2024-10-10 15:18:42","2024-10-10 16:18:42"]}}"#;

        let node: Node = serde_json::from_str(json_str)?;
        let backtest_spot_ticker = BacktestSpotTicker::try_from(node)?;

        assert_eq!(backtest_spot_ticker.params.base_asset, "BTC");
        assert_eq!(backtest_spot_ticker.params.quote_asset, "USDT");
        assert_eq!(
            backtest_spot_ticker.params.start_datetime,
            add_utc_offset("2024-10-10 15:18:42")?
        );
        assert_eq!(
            backtest_spot_ticker.params.end_datetime,
            add_utc_offset("2024-10-10 16:18:42")?
        );

        Ok(())
    }
}
