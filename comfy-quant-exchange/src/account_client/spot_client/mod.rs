mod base;
mod mock_spot_client;

use anyhow::Result;
use base::{AccountInformation, Balance, Order};
use enum_dispatch::enum_dispatch;
use mock_spot_client::MockSpotClient;

#[enum_dispatch]
pub trait SpotClient {
    // 获取账户信息，手续费
    async fn get_account(&self) -> Result<AccountInformation>;

    // 获取账户余额
    async fn get_balance(&self, asset: &str) -> Result<Balance>;

    // 获取订单信息
    async fn get_order(&self, order_id: &str) -> Result<Order>;

    // 市价买单
    async fn market_buy(&self, symbol: &str, qty: f64) -> Result<Order>;

    // 市价卖单
    async fn market_sell(&self, symbol: &str, qty: f64) -> Result<Order>;

    // 限价买单
    async fn limit_buy(&self, symbol: &str, qty: f64, price: f64) -> Result<Order>;

    // 限价卖单
    async fn limit_sell(&self, symbol: &str, qty: f64, price: f64) -> Result<Order>;
}

#[enum_dispatch(SpotClient)]
pub enum SpotClientEnum {
    MockSpotClient(MockSpotClient),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_spot_client_enum() -> Result<()> {
        let client: SpotClientEnum = MockSpotClient::new().into();
        let account = client.get_account().await?;
        assert_eq!(account.maker_commission, 0.001);
        assert_eq!(account.taker_commission, 0.001);
        Ok(())
    }
}
