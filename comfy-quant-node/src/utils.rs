use tokio::sync::mpsc::{self, UnboundedSender};
use tokio_stream::wrappers::UnboundedReceiverStream;

pub fn create_mpsc_channel<T>() -> (UnboundedSender<T>, UnboundedReceiverStream<T>) {
    let (tx, rx) = mpsc::unbounded_channel();
    (tx, UnboundedReceiverStream::new(rx))
}
