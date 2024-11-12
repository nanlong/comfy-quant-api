use super::{
    base::{SpotClientRequest, SpotClientResponse},
    binance_spot_client::BinanceSpotClient,
};
use crate::client::spot_client_kind::SpotClientExecutable;
use binance::config::Config;
use bon::bon;
use futures::future::BoxFuture;
use std::task::{Context, Poll};
use tower::Service;

pub struct BinanceSpotClientService {
    client: BinanceSpotClient,
}

impl From<&BinanceSpotClient> for BinanceSpotClientService {
    fn from(value: &BinanceSpotClient) -> Self {
        BinanceSpotClientService::builder()
            .client(value.clone())
            .build()
    }
}

impl From<BinanceSpotClient> for BinanceSpotClientService {
    fn from(value: BinanceSpotClient) -> Self {
        BinanceSpotClientService::builder().client(value).build()
    }
}

#[bon]
impl BinanceSpotClientService {
    #[builder(on(String, into))]
    pub fn new(
        client: Option<BinanceSpotClient>,
        api_key: Option<String>,
        secret_key: Option<String>,
        config: Option<Config>,
    ) -> Self {
        let client = client.unwrap_or_else(|| {
            BinanceSpotClient::builder()
                .maybe_api_key(api_key)
                .maybe_secret_key(secret_key)
                .maybe_config(config)
                .build()
        });

        BinanceSpotClientService { client }
    }
}

impl Service<SpotClientRequest> for BinanceSpotClientService {
    type Response = SpotClientResponse;
    type Error = anyhow::Error;
    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: SpotClientRequest) -> Self::Future {
        let client = self.client.clone();

        let fut = async move {
            let res = match req {
                SpotClientRequest::PlatformName => client.platform_name().into(),
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
