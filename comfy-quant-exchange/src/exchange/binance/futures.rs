use super::BinanceClient;
use anyhow::{anyhow, Result};
use binance::{
    api::Binance,
    futures::{
        account::{FuturesAccount as Account, TimeInForce},
        general::FuturesGeneral as General,
        market::FuturesMarket as Market,
        model::{
            AccountBalance, AccountInformation, ExchangeInformation, OrderBook, Symbol, Transaction,
        },
    },
    model::{KlineSummaries, SymbolPrice},
};

pub struct Futures<'a> {
    client: &'a BinanceClient,
}

impl<'a> Futures<'a> {
    pub fn new(client: &'a BinanceClient) -> Self {
        Futures { client }
    }

    fn account(&self) -> Account {
        self.client
            .create_api(Account::new, Account::new_with_config)
    }

    fn market(&self) -> Market {
        self.client.create_api(Market::new, Market::new_with_config)
    }

    fn general(&self) -> General {
        self.client
            .create_api(General::new, General::new_with_config)
    }

    pub fn ping(&self) -> Result<String> {
        let ping = self.general().ping().map_err(|e| anyhow!(e.to_string()))?;

        Ok(ping)
    }

    pub fn get_exchange_info(&self) -> Result<ExchangeInformation> {
        let exchange_info = self
            .general()
            .exchange_info()
            .map_err(|e| anyhow!(e.to_string()))?;

        Ok(exchange_info)
    }

    pub fn get_symbol_info(&self, symbol: impl Into<String>) -> Result<Symbol> {
        let symbol_info = self
            .general()
            .get_symbol_info(symbol)
            .map_err(|e| anyhow!(e.to_string()))?;

        Ok(symbol_info)
    }

    // 获取账户信息
    pub fn get_account(&self) -> Result<AccountInformation> {
        let account_information = self
            .account()
            .account_information()
            .map_err(|e| anyhow!(e.to_string()))?;

        Ok(account_information)
    }

    // 获取资产
    pub fn get_asset(&self, asset: impl Into<String>) -> Result<AccountBalance> {
        let asset = asset.into();

        let balances = self
            .account()
            .account_balance()
            .map_err(|e| anyhow!(e.to_string()))?;

        let balance = balances
            .iter()
            .find(|b| b.asset == asset)
            .ok_or_else(|| anyhow!("Asset not found"))?
            .clone();

        Ok(balance)
    }

    // 限价买入
    pub fn limit_buy(
        &self,
        symbol: impl Into<String>,  // 交易对
        qty: impl Into<f64>,        // 数量
        price: f64,                 // 价格
        time_in_force: TimeInForce, // 时间限制
    ) -> Result<Transaction> {
        let transaction = self
            .account()
            .limit_buy(symbol, qty, price, time_in_force)
            .map_err(|e| anyhow!(e.to_string()))?;

        Ok(transaction)
    }

    // 限价卖出
    pub fn limit_sell(
        &self,
        symbol: impl Into<String>,  // 交易对
        qty: impl Into<f64>,        // 数量
        price: f64,                 // 价格
        time_in_force: TimeInForce, // 时间限制
    ) -> Result<Transaction> {
        let transaction = self
            .account()
            .limit_sell(symbol, qty, price, time_in_force)
            .map_err(|e| anyhow!(e.to_string()))?;

        Ok(transaction)
    }

    // 市价买入
    pub fn market_buy(
        &self,
        symbol: impl Into<String>, // 交易对
        qty: impl Into<f64>,       // 数量
    ) -> Result<Transaction> {
        let transaction = self
            .account()
            .market_buy(symbol, qty)
            .map_err(|e| anyhow!(e.to_string()))?;

        Ok(transaction)
    }

    // 市价卖出
    pub fn market_sell(
        &self,
        symbol: impl Into<String>, // 交易对
        qty: impl Into<f64>,       // 数量
    ) -> Result<Transaction> {
        let transaction = self
            .account()
            .market_sell(symbol, qty)
            .map_err(|e| anyhow!(e.to_string()))?;

        Ok(transaction)
    }

    // 获取价格
    pub fn get_price(&self, symbol: impl Into<String>) -> Result<SymbolPrice> {
        let price = self
            .market()
            .get_price(symbol)
            .map_err(|e| anyhow!(e.to_string()))?;

        Ok(price)
    }

    // 获取深度
    pub fn get_depth(&self, symbol: impl Into<String>) -> Result<OrderBook> {
        let order_book = self
            .market()
            .get_depth(symbol)
            .map_err(|e| anyhow!(e.to_string()))?;

        Ok(order_book)
    }

    // 获取K线
    pub fn get_klines(
        &self,
        symbol: impl Into<String>,          // 交易对
        interval: impl Into<String>,        // 时间间隔
        limit: impl Into<Option<u16>>,      // 限制数量
        start_time: impl Into<Option<u64>>, // 开始时间
        end_time: impl Into<Option<u64>>,   // 结束时间
    ) -> Result<KlineSummaries> {
        let klines = self
            .market()
            .get_klines(symbol, interval, limit, start_time, end_time)
            .map_err(|e| anyhow!(e.to_string()))?;

        Ok(klines)
    }
}
