use std::{
    pin::Pin,
    task::{Context, Poll},
};

use anyhow::Result;
use bon::Builder;
use flume::{Receiver, Sender};
use futures::Stream;

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
        let (sender, receiver) = flume::unbounded();
        Self {
            channel: (sender, receiver),
        }
    }

    pub async fn send(&self, tick: Tick) -> Result<()> {
        let (tx, _rx) = &self.channel;
        tx.send_async(tick).await?;
        Ok(())
    }
}

impl Stream for TickStream {
    type Item = Tick;

    fn poll_next(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let (_tx, rx) = &self.channel;

        match rx.recv() {
            Ok(tick) => Poll::Ready(Some(tick)),
            Err(_) => Poll::Ready(None),
        }
    }
}

#[cfg(test)]
mod tests {
    use futures::StreamExt;

    use super::*;

    #[tokio::test]
    async fn test_tick_stream() -> Result<()> {
        let mut tick_stream = TickStream::new();
        let tick = Tick {
            timestamp: 1,
            price: 100.0,
        };

        tick_stream.send(tick.clone()).await?;

        let tick2 = tick_stream.next().await.unwrap();
        assert_eq!(tick, tick2);

        Ok(())
    }
}
