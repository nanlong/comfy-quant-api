use super::spot_client::{
    backtest_spot_client::BacktestSpotClient,
    base::{AccountInformation, Balance, Order, SymbolInformation, SymbolPrice},
    binance_spot_client::BinanceSpotClient,
};
use anyhow::Result;
use enum_dispatch::enum_dispatch;

#[enum_dispatch]
#[allow(async_fn_in_trait)]
pub trait SpotClientExecutable {
    fn platform_name(&self) -> String;

    // 获取账户信息，手续费
    async fn get_account(&self) -> Result<AccountInformation>;

    async fn get_symbol_info(
        &self,
        base_asset: &str,
        quote_asset: &str,
    ) -> Result<SymbolInformation>;

    // 获取账户余额
    async fn get_balance(&self, asset: &str) -> Result<Balance>;

    // 获取订单信息
    async fn get_order(&self, base_asset: &str, quote_asset: &str, order_id: &str)
        -> Result<Order>;

    // 市价买单
    async fn market_buy(&self, base_asset: &str, quote_asset: &str, qty: f64) -> Result<Order>;

    // 市价卖单
    async fn market_sell(&self, base_asset: &str, quote_asset: &str, qty: f64) -> Result<Order>;

    // 限价买单
    async fn limit_buy(
        &self,
        base_asset: &str,
        quote_asset: &str,
        qty: f64,
        price: f64,
    ) -> Result<Order>;

    // 限价卖单
    async fn limit_sell(
        &self,
        base_asset: &str,
        quote_asset: &str,
        qty: f64,
        price: f64,
    ) -> Result<Order>;

    // 获取价格
    async fn get_price(&self, base_asset: &str, quote_asset: &str) -> Result<SymbolPrice>;
}

#[derive(Debug, Clone)]
#[enum_dispatch(SpotClientExecutable)]
pub enum SpotClientKind {
    BacktestSpotClient(BacktestSpotClient),
    BinanceSpotClient(BinanceSpotClient),
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[tokio::test]
    async fn test_spot_client_enum() -> Result<()> {
        let client: SpotClientKind = BacktestSpotClient::builder()
            .assets(vec![("BTC".to_string(), 1.), ("USDT".to_string(), 1000.)])
            .build()
            .into();
        let account = client.get_account().await?;
        assert_eq!(account.maker_commission_rate, dec!(0.001));
        assert_eq!(account.taker_commission_rate, dec!(0.001));
        Ok(())
    }
}
