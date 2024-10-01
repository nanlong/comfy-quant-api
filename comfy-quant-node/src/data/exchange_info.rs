use bon::Builder;

#[derive(Debug, Clone, Builder)]
#[builder(on(String, into))]
pub struct ExchangeInfo {
    pub name: String,
    pub market: String,
    pub base_currency: String,
    pub quote_currency: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exchange_info_builder() {
        let exchange = ExchangeInfo::builder()
            .name("binance")
            .market("spot")
            .base_currency("BTC")
            .quote_currency("USDT")
            .build();

        assert_eq!(exchange.name, "binance");
        assert_eq!(exchange.market, "spot");
        assert_eq!(exchange.base_currency, "BTC");
        assert_eq!(exchange.quote_currency, "USDT");
    }
}
