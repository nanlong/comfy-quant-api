use crate::{
    node_core::{NodeExecutable, NodeInfra, NodePort, Port, Slot, Tick},
    node_io::{SpotPairInfo, TickStream},
    workflow::Node,
};
use anyhow::Result;
use bon::Builder;
use chrono::{DateTime, Utc};
use comfy_quant_base::{convert_to_datetime, Exchange, KlineInterval, Market};
use comfy_quant_database::kline::{self};
use comfy_quant_exchange::client::spot_client::base::BINANCE_EXCHANGE_NAME;
use comfy_quant_task::{
    task_core::{status::TaskStatus, traits::Executable as _},
    tasks::binance_klines::BinanceKlinesTask,
};
use futures::StreamExt;
use std::sync::Arc;

/// 回测行情数据
/// outputs:
///      0: SpotPairInfo
///      1: TickStream
#[derive(Debug)]
#[allow(unused)]
pub(crate) struct BacktestSpotTicker {
    params: Params,
    infra: NodeInfra,
    exchange: Exchange,
    market: Market,
    interval: KlineInterval,
}

impl BacktestSpotTicker {
    pub(crate) fn try_new(node: Node) -> Result<Self> {
        let params = Params::try_from(&node)?;
        let mut infra = NodeInfra::new(node);

        let pair_info = SpotPairInfo::builder()
            .base_asset(&params.base_asset)
            .quote_asset(&params.quote_asset)
            .build();
        let tick_stream = TickStream::new();

        let pair_info_slot = Arc::new(Slot::<SpotPairInfo>::new(pair_info));
        let tick_stream_slot = Arc::new(Slot::<TickStream>::new(tick_stream));

        infra.port.set_output(0, pair_info_slot)?;
        infra.port.set_output(1, tick_stream_slot)?;

        Ok(BacktestSpotTicker {
            params,
            infra,
            exchange: BINANCE_EXCHANGE_NAME.into(),
            market: Market::Spot,
            interval: KlineInterval::OneSecond,
        })
    }

    async fn output1(&self) -> Result<()> {
        let port = self.port();
        let slot1 = port.output::<TickStream>(1)?;
        let symbol =
            format!("{}{}", self.params.base_asset, self.params.quote_asset).to_uppercase();
        let start_timestamp = self.params.start_datetime.timestamp();
        let end_timestamp = self.params.end_datetime.timestamp();
        let ctx = self.infra.node_context()?;
        let db = ctx.db.clone();

        // 等待数据同步完成，如果出错，重试3次
        'retry: for i in 0..3 {
            let task = BinanceKlinesTask::builder()
                .db(Arc::clone(&db))
                .market(self.market.as_ref())
                .symbol(&symbol)
                .interval(self.interval.as_ref())
                .start_timestamp(start_timestamp)
                .end_timestamp(end_timestamp)
                .build()?;

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
            self.exchange.as_ref(),
            self.market.as_ref(),
            &symbol,
            self.interval.as_ref(),
            &self.params.start_datetime,
            &self.params.end_datetime,
        );

        let price_store = self.infra.workflow_context()?.cloned_price_store();

        while let Some(Ok(kline)) = klines_stream.next().await {
            let tick = Tick::builder()
                .timestamp(kline.open_time.timestamp())
                .symbol(symbol.clone())
                .price(kline.close_price)
                .build();

            price_store.write().await.save_price(
                self.exchange.clone(),
                self.market.clone(),
                tick.clone().into(),
            )?;

            slot1
                .send(self.exchange.as_ref(), self.market.as_ref(), tick)
                .await?;
        }

        Ok(())
    }
}

impl NodePort for BacktestSpotTicker {
    fn port(&self) -> &Port {
        &self.infra.port
    }

    fn port_mut(&mut self) -> &mut Port {
        &mut self.infra.port
    }
}

impl NodeExecutable for BacktestSpotTicker {
    async fn execute(&mut self) -> Result<()> {
        // 同步等待其他节点
        self.infra.workflow_context()?.wait().await;

        self.output1().await?;
        Ok(())
    }
}

impl TryFrom<Node> for BacktestSpotTicker {
    type Error = anyhow::Error;

    fn try_from(node: Node) -> Result<Self> {
        BacktestSpotTicker::try_new(node)
    }
}

impl TryFrom<&BacktestSpotTicker> for Node {
    type Error = anyhow::Error;

    fn try_from(backtest_spot_ticker: &BacktestSpotTicker) -> Result<Self> {
        Ok(backtest_spot_ticker.infra.node.clone())
    }
}

#[derive(Builder, Debug, Clone)]
#[builder(on(String, into))]
pub(crate) struct Params {
    base_asset: String,
    quote_asset: String,
    start_datetime: DateTime<Utc>,
    end_datetime: DateTime<Utc>,
}

impl TryFrom<&Node> for Params {
    type Error = BacktestSpotTickerError;

    fn try_from(node: &Node) -> Result<Self, Self::Error> {
        if node.properties.prop_type != "data.BacktestSpotTicker" {
            return Err(BacktestSpotTickerError::PropertyTypeMismatch);
        }

        let [base_asset, quote_asset, start_datetime, end_datetime] =
            node.properties.params.as_slice()
        else {
            return Err(BacktestSpotTickerError::ParamsFormatError);
        };

        let base_asset = base_asset
            .as_str()
            .ok_or(BacktestSpotTickerError::BaseAssetError)?;

        let quote_asset = quote_asset
            .as_str()
            .ok_or(BacktestSpotTickerError::QuoteAssetError)?;

        let start_datetime = start_datetime
            .as_str()
            .and_then(convert_to_datetime)
            .ok_or(BacktestSpotTickerError::StartDatetimeError)?;

        let end_datetime = end_datetime
            .as_str()
            .and_then(convert_to_datetime)
            .ok_or(BacktestSpotTickerError::EndDatetimeError)?;

        let params = Params::builder()
            .base_asset(base_asset)
            .quote_asset(quote_asset)
            .start_datetime(start_datetime)
            .end_datetime(end_datetime)
            .build();

        Ok(params)
    }
}

#[derive(thiserror::Error, Debug)]
pub enum BacktestSpotTickerError {
    #[error("Invalid property type, expected 'data.BacktestSpotTicker'")]
    PropertyTypeMismatch,

    #[error("Invalid parameters format")]
    ParamsFormatError,

    #[error("Invalid base asset")]
    BaseAssetError,

    #[error("Invalid quote asset")]
    QuoteAssetError,

    #[error("Invalid start datetime")]
    StartDatetimeError,

    #[error("Invalid end datetime")]
    EndDatetimeError,
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
            convert_to_datetime("2024-10-10 15:18:42").unwrap()
        );
        assert_eq!(
            backtest_spot_ticker.params.end_datetime,
            convert_to_datetime("2024-10-10 16:18:42").unwrap()
        );

        Ok(())
    }
}
