use anyhow::Result;
use bon::Builder;
use flume::{Receiver, Sender};
use rust_decimal::Decimal;

#[derive(Debug, Clone, Builder, PartialEq)]
pub(crate) struct Tick {
    pub(crate) timestamp: i64,
    pub(crate) price: Decimal,
}

#[derive(Debug, Clone, Builder)]
pub(crate) struct TickStream {
    pub(crate) channel: (Sender<Tick>, Receiver<Tick>),
}

impl TickStream {
    pub(crate) fn new() -> Self {
        let channel = flume::unbounded();
        TickStream { channel }
    }

    pub(crate) async fn send(&self, tick: Tick) -> Result<()> {
        self.channel.0.send_async(tick).await?;
        Ok(())
    }

    pub(crate) fn subscribe(&self) -> Receiver<Tick> {
        self.channel.1.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_tick_stream() -> Result<()> {
        let tick_stream = TickStream::new();
        let tick = Tick {
            timestamp: 1,
            price: Decimal::try_from(100.0).unwrap(),
        };

        tick_stream.send(tick.clone()).await?;

        let rx = tick_stream.subscribe();

        let tick2 = rx.recv_async().await?;
        assert_eq!(tick, tick2);

        Ok(())
    }
}
