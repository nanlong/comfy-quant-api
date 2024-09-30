pub struct Token {
    symbol: String,
    amount: f64,
}

pub struct AccountManager {
    // 总投入
    total_investment: f64,
    // 基础货币
    base_currency: Token,
    // 报价货币
    quote_currency: Token,
}

impl AccountManager {
    pub fn new(total_investment: f64, base_currency: Token, quote_currency: Token) -> Self {
        Self {
            total_investment,
            base_currency,
            quote_currency,
        }
    }
}
