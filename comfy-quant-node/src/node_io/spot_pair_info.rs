use bon::Builder;

#[derive(Debug, Clone, Builder)]
#[builder(on(String, into))]
#[allow(unused)]
pub(crate) struct SpotPairInfo {
    pub(crate) base_currency: String,
    pub(crate) quote_currency: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exchange_info_builder() {
        let exchange = SpotPairInfo::builder()
            .base_currency("BTC")
            .quote_currency("USDT")
            .build();

        assert_eq!(exchange.base_currency, "BTC");
        assert_eq!(exchange.quote_currency, "USDT");
    }
}
