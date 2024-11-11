use super::base::{AccountInformation, Balance, Order, SymbolInformation, SymbolPrice};
use crate::{client::spot_client_kind::SpotClientExecutable, exchange::binance::BinanceClient};
use anyhow::Result;
use binance::config::Config;
use bon::bon;
use tokio::task::spawn_blocking;

#[derive(Debug, Clone)]
pub struct BinanceSpotClient {
    client: BinanceClient,
}

#[bon]
impl BinanceSpotClient {
    #[builder(on(String, into))]
    pub fn new(
        api_key: Option<String>,
        secret_key: Option<String>,
        config: Option<Config>,
    ) -> Self {
        let client = BinanceClient::builder()
            .maybe_api_key(api_key)
            .maybe_secret_key(secret_key)
            .maybe_config(config)
            .build();

        BinanceSpotClient { client }
    }

    fn symbol(base_asset: &str, quote_asset: &str) -> String {
        format!(
            "{}{}",
            base_asset.to_uppercase(),
            quote_asset.to_uppercase()
        )
    }

    pub async fn ping(&self) -> Result<String> {
        let client = self.client.clone();
        let pong = spawn_blocking(move || client.spot().ping()).await??;
        Ok(pong)
    }
}

impl SpotClientExecutable for BinanceSpotClient {
    fn platform_name(&self) -> String {
        "Binance".to_string()
    }

    async fn get_account(&self) -> Result<AccountInformation> {
        let client = self.client.clone();
        let account = spawn_blocking(move || client.spot().get_account()).await??;

        account.try_into()
    }

    async fn get_symbol_info(
        &self,
        base_asset: &str,
        quote_asset: &str,
    ) -> Result<SymbolInformation> {
        let client = self.client.clone();
        let symbol = Self::symbol(base_asset, quote_asset);
        let symbol = spawn_blocking(move || client.spot().get_symbol_info(symbol)).await??;

        dbg!(&symbol);

        Ok(symbol.into())
    }

    async fn get_balance(&self, asset: &str) -> Result<Balance> {
        let client = self.client.clone();
        let asset = asset.to_uppercase();
        let balance = spawn_blocking(move || client.spot().get_balance(asset)).await??;

        Ok(balance.into())
    }

    async fn get_order(
        &self,
        base_asset: &str,
        quote_asset: &str,
        order_id: &str,
    ) -> Result<Order> {
        let client = self.client.clone();
        let symbol = Self::symbol(base_asset, quote_asset);
        let order_id = order_id.parse::<u64>()?;
        let order = spawn_blocking(move || client.spot().get_order(symbol, order_id)).await??;

        order.try_into()
    }

    async fn market_buy(&self, base_asset: &str, quote_asset: &str, qty: f64) -> Result<Order> {
        let client = self.client.clone();
        let symbol = Self::symbol(base_asset, quote_asset);
        let tx = spawn_blocking(move || client.spot().market_buy(symbol, qty)).await??;
        dbg!(&tx);
        tx.try_into()
    }

    async fn market_sell(&self, base_asset: &str, quote_asset: &str, qty: f64) -> Result<Order> {
        let client = self.client.clone();
        let symbol = Self::symbol(base_asset, quote_asset);
        let tx = spawn_blocking(move || client.spot().market_sell(symbol, qty)).await??;

        tx.try_into()
    }

    async fn limit_buy(
        &self,
        base_asset: &str,
        quote_asset: &str,
        qty: f64,
        price: f64,
    ) -> Result<Order> {
        let client = self.client.clone();
        let symbol = Self::symbol(base_asset, quote_asset);
        let tx = spawn_blocking(move || client.spot().limit_buy(symbol, qty, price)).await??;

        tx.try_into()
    }

    async fn limit_sell(
        &self,
        base_asset: &str,
        quote_asset: &str,
        qty: f64,
        price: f64,
    ) -> Result<Order> {
        let client = self.client.clone();
        let symbol = Self::symbol(base_asset, quote_asset);
        let tx = spawn_blocking(move || client.spot().limit_sell(symbol, qty, price)).await??;

        tx.try_into()
    }

    async fn get_price(&self, base_asset: &str, quote_asset: &str) -> Result<SymbolPrice> {
        let client = self.client.clone();
        let symbol = Self::symbol(base_asset, quote_asset);
        let symbol_price = spawn_blocking(move || client.spot().get_price(symbol)).await??;

        symbol_price.try_into()
    }
}
