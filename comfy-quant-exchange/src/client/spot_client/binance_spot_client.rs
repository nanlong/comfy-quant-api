use super::base::{
    AccountInformation, Balance, BinanceOrder, BinanceTransaction, Order, SymbolInformation,
    SymbolPrice,
};
use crate::{client::spot_client_kind::SpotClientExecutable, exchange::binance::BinanceClient};
use anyhow::Result;
use binance::config::Config;
use bon::bon;
use comfy_quant_base::{Exchange, Symbol};

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
}

impl SpotClientExecutable for BinanceSpotClient {
    fn exchange(&self) -> Exchange {
        Exchange::Binance
    }

    fn symbol(&self, base_asset: &str, quote_asset: &str) -> Symbol {
        format!(
            "{}{}",
            base_asset.to_uppercase(),
            quote_asset.to_uppercase()
        )
        .into()
    }

    async fn get_account(&self) -> Result<AccountInformation> {
        self.client.spot().get_account()?.try_into()
    }

    async fn get_symbol_info(
        &self,
        base_asset: &str,
        quote_asset: &str,
    ) -> Result<SymbolInformation> {
        let symbol = self.symbol(base_asset, quote_asset);
        let symbol_info = self.client.spot().get_symbol_info(symbol)?;
        Ok(symbol_info.into())
    }

    async fn get_balance(&self, asset: &str) -> Result<Balance> {
        let asset = asset.to_uppercase();
        let balance = self.client.spot().get_balance(asset)?;
        Ok(balance.into())
    }

    async fn get_order(
        &self,
        base_asset: &str,
        quote_asset: &str,
        order_id: &str,
    ) -> Result<Order> {
        let symbol = self.symbol(base_asset, quote_asset);
        let order = self.client.spot().get_order(symbol, order_id.parse()?)?;

        BinanceOrder::builder()
            .base_asset(base_asset)
            .quote_asset(quote_asset)
            .order(order)
            .build()
            .try_into()
    }

    async fn market_buy(&self, base_asset: &str, quote_asset: &str, qty: f64) -> Result<Order> {
        let symbol = self.symbol(base_asset, quote_asset);
        let tx = self.client.spot().market_buy(symbol, qty)?;

        BinanceTransaction::builder()
            .base_asset(base_asset)
            .quote_asset(quote_asset)
            .transaction(tx)
            .build()
            .try_into()
    }

    async fn market_sell(&self, base_asset: &str, quote_asset: &str, qty: f64) -> Result<Order> {
        let symbol = self.symbol(base_asset, quote_asset);
        let tx = self.client.spot().market_sell(symbol, qty)?;

        BinanceTransaction::builder()
            .base_asset(base_asset)
            .quote_asset(quote_asset)
            .transaction(tx)
            .build()
            .try_into()
    }

    async fn limit_buy(
        &self,
        base_asset: &str,
        quote_asset: &str,
        qty: f64,
        price: f64,
    ) -> Result<Order> {
        let symbol = self.symbol(base_asset, quote_asset);
        let tx = self.client.spot().limit_buy(symbol, qty, price)?;

        BinanceTransaction::builder()
            .base_asset(base_asset)
            .quote_asset(quote_asset)
            .transaction(tx)
            .build()
            .try_into()
    }

    async fn limit_sell(
        &self,
        base_asset: &str,
        quote_asset: &str,
        qty: f64,
        price: f64,
    ) -> Result<Order> {
        let symbol = self.symbol(base_asset, quote_asset);
        let tx = self.client.spot().limit_sell(symbol, qty, price)?;

        BinanceTransaction::builder()
            .base_asset(base_asset)
            .quote_asset(quote_asset)
            .transaction(tx)
            .build()
            .try_into()
    }

    async fn get_price(&self, base_asset: &str, quote_asset: &str) -> Result<SymbolPrice> {
        let symbol = self.symbol(base_asset, quote_asset);
        self.client.spot().get_price(symbol)?.try_into()
    }
}
