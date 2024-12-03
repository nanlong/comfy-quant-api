use super::spot_client::{
    backtest_spot_client::BacktestSpotClient,
    base::{
        AccountInformation, Balance, Order, SpotClientRequest, SpotClientResponse,
        SymbolInformation, SymbolPrice,
    },
    binance_spot_client::BinanceSpotClient,
};
use anyhow::Result;
use comfy_quant_base::Exchange;
use enum_dispatch::enum_dispatch;
use futures::future::BoxFuture;
use std::task::{Context, Poll};
use tower::Service;

#[enum_dispatch]
#[allow(async_fn_in_trait)]
pub trait SpotClientExecutable {
    fn exchange(&self) -> Exchange;

    fn symbol(&self, base_asset: &str, quote_asset: &str) -> String;

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

impl Service<SpotClientRequest> for SpotClientKind {
    type Response = SpotClientResponse;
    type Error = anyhow::Error;
    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: SpotClientRequest) -> Self::Future {
        let client = self.clone();

        let fut = async move {
            let res = match req {
                SpotClientRequest::Exchange => {
                    let name = client.exchange().to_string();
                    SpotClientResponse::Exchange(name)
                }
                SpotClientRequest::Symbol {
                    base_asset,
                    quote_asset,
                } => {
                    let symbol = client.symbol(&base_asset, &quote_asset);
                    SpotClientResponse::Symbol(symbol)
                }
                SpotClientRequest::GetAccount => client.get_account().await?.into(),
                SpotClientRequest::GetSymbolInfo {
                    base_asset,
                    quote_asset,
                } => client
                    .get_symbol_info(&base_asset, &quote_asset)
                    .await?
                    .into(),
                SpotClientRequest::GetBalance { asset } => client.get_balance(&asset).await?.into(),
                SpotClientRequest::GetOrder {
                    base_asset,
                    quote_asset,
                    order_id,
                } => client
                    .get_order(&base_asset, &quote_asset, &order_id)
                    .await?
                    .into(),
                SpotClientRequest::MarketBuy {
                    base_asset,
                    quote_asset,
                    qty,
                } => client
                    .market_buy(&base_asset, &quote_asset, qty)
                    .await?
                    .into(),
                SpotClientRequest::MarketSell {
                    base_asset,
                    quote_asset,
                    qty,
                } => client
                    .market_sell(&base_asset, &quote_asset, qty)
                    .await?
                    .into(),
                SpotClientRequest::LimitBuy {
                    base_asset,
                    quote_asset,
                    qty,
                    price,
                } => client
                    .limit_buy(&base_asset, &quote_asset, qty, price)
                    .await?
                    .into(),
                SpotClientRequest::LimitSell {
                    base_asset,
                    quote_asset,
                    qty,
                    price,
                } => client
                    .limit_sell(&base_asset, &quote_asset, qty, price)
                    .await?
                    .into(),
                SpotClientRequest::GetPrice {
                    base_asset,
                    quote_asset,
                } => client.get_price(&base_asset, &quote_asset).await?.into(),
            };

            Ok(res)
        };

        Box::pin(fut)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::store::PriceStore;
    use async_lock::RwLock;
    use rust_decimal_macros::dec;
    use std::sync::Arc;

    #[tokio::test]
    async fn test_spot_client_enum() -> Result<()> {
        let client: SpotClientKind = BacktestSpotClient::builder()
            .assets(vec![("BTC".to_string(), 1.), ("USDT".to_string(), 1000.)])
            .price_store(Arc::new(RwLock::new(PriceStore::new())))
            .build()
            .into();
        let account = client.get_account().await?;
        assert_eq!(account.maker_commission_rate, dec!(0.001));
        assert_eq!(account.taker_commission_rate, dec!(0.001));
        Ok(())
    }
}
