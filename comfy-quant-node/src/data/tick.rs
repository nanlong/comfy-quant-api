use bon::Builder;

#[derive(Debug, Clone, Builder)]
pub struct Tick {
    pub timestamp: i64,
    pub price: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ticker_builder() {
        let tick = Tick::builder().timestamp(1672531200).price(60000.0).build();

        assert_eq!(tick.timestamp, 1672531200);
        assert_eq!(tick.price, 60000.0);
    }
}
