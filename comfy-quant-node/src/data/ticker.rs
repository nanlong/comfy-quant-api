use bon::Builder;

#[derive(Debug, Clone, Builder)]
pub struct Ticker {
    pub timestamp: i64,
    pub price: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ticker_builder() {
        let ticker = Ticker::builder()
            .timestamp(1672531200)
            .price(60000.0)
            .build();

        assert_eq!(ticker.timestamp, 1672531200);
        assert_eq!(ticker.price, 60000.0);
    }
}
