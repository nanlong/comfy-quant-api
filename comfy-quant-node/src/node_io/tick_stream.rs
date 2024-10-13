use anyhow::Result;
use bon::Builder;
use flume::{Receiver, Sender};

#[derive(Debug, Clone, Builder, PartialEq)]
pub struct Tick {
    pub timestamp: i64,
    pub price: f64,
}

#[derive(Debug, Clone, Builder)]
pub struct TickStream {
    pub(crate) channel: (Sender<Tick>, Receiver<Tick>),
}

impl TickStream {
    pub fn new() -> Self {
        let channel = flume::unbounded();
        TickStream { channel }
    }

    pub async fn send(&self, tick: Tick) -> Result<()> {
        self.channel.0.send_async(tick).await?;
        Ok(())
    }

    pub fn subscribe(&self) -> Receiver<Tick> {
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
            price: 100.0,
        };

        tick_stream.send(tick.clone()).await?;

        let rx = tick_stream.subscribe();

        let tick2 = rx.recv_async().await?;
        assert_eq!(tick, tick2);

        Ok(())
    }
}
