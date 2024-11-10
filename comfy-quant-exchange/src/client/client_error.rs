use binance::errors::{Error as BinanceError, ErrorKind as BinanceErrorKind};
use thiserror::Error;

#[derive(Error, Debug)]
pub(crate) enum ClientError {
    #[error("binance error: {}", fetch_binance_error(.0))]
    BinanceError(#[from] BinanceError),
}

fn fetch_binance_error(error: &BinanceError) -> String {
    match error.0 {
        BinanceErrorKind::BinanceError(ref binance_content_error) => {
            binance_content_error.msg.clone()
        }
        _ => error.0.to_string(),
    }
}

unsafe impl Send for ClientError {}
unsafe impl Sync for ClientError {}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;
    use binance::errors::{BinanceContentError, ErrorKind};

    #[test]
    fn test_client_error() -> Result<()> {
        let binance_content_error = BinanceContentError {
            code: 0,
            msg: "test".to_string(),
        };
        let kind = ErrorKind::BinanceError(binance_content_error);
        let error = BinanceError::from_kind(kind);
        let client_error = ClientError::BinanceError(error);

        assert_eq!(client_error.to_string(), "binance error: test");

        Ok(())
    }
}
