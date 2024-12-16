use crate::node_core::Tick;
use anyhow::Result;
use bon::Builder;
use comfy_quant_base::{Exchange, Market};
use flume::{Receiver, Sender};
use tokio_util::sync::CancellationToken;

type ExchangeTick = (Exchange, Market, Tick);

#[derive(Debug, Builder)]
pub(crate) struct TickStream {
    inner: (Sender<ExchangeTick>, Receiver<ExchangeTick>),
    token: CancellationToken,
}

impl TickStream {
    pub(crate) fn new() -> Self {
        TickStream {
            inner: flume::unbounded(),
            token: CancellationToken::new(),
        }
    }

    pub(crate) async fn send(
        &self,
        exchange: &Exchange,
        market: &Market,
        tick: &Tick,
    ) -> Result<()> {
        self.inner
            .0
            .send_async((exchange.clone(), market.clone(), tick.clone()))
            .await?;
        Ok(())
    }

    pub(crate) fn subscribe(&self) -> Receiver<ExchangeTick> {
        self.inner.1.clone()
    }
}

impl Drop for TickStream {
    fn drop(&mut self) {
        self.token.cancel();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[tokio::test]
    async fn test_tick_stream() -> Result<()> {
        let tick_stream = TickStream::new();
        let tick = Tick {
            timestamp: 1,
            symbol: "BTCUSDT".into(),
            price: dec!(100.0),
        };
        let exchange = Exchange::Binance;
        let market = Market::Spot;

        tick_stream.send(&exchange, &market, &tick).await?;

        let rx = tick_stream.subscribe();

        let tick2 = rx.recv_async().await?;
        assert_eq!((exchange, market, tick), tick2);

        Ok(())
    }
}
