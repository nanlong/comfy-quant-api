use bon::Builder;

#[derive(Debug, Clone, Builder)]
#[builder(on(String, into))]
#[allow(unused)]
pub(crate) struct SpotPairInfo {
    pub(crate) base_asset: String,
    pub(crate) quote_asset: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exchange_info_builder() {
        let exchange = SpotPairInfo::builder()
            .base_asset("BTC")
            .quote_asset("USDT")
            .build();

        assert_eq!(exchange.base_asset, "BTC");
        assert_eq!(exchange.quote_asset, "USDT");
    }
}
