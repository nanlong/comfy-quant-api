#[derive(Debug, Clone)]
pub struct ExchangeInfo {
    pub name: String,
    pub market: String,
    pub base_currency: String,
    pub quote_currency: String,
}

impl ExchangeInfo {
    pub fn new(
        name: impl Into<String>,
        market: impl Into<String>,
        base_currency: impl Into<String>,
        quote_currency: impl Into<String>,
    ) -> Self {
        Self {
            name: name.into(),
            market: market.into(),
            base_currency: base_currency.into(),
            quote_currency: quote_currency.into(),
        }
    }
}
