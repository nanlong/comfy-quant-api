use super::BinanceClient;
use anyhow::{anyhow, Result};
use binance::{
    account::Account,
    api::Binance,
    general::General,
    market::Market,
    model::{
        AccountInformation, Balance, ExchangeInformation, KlineSummaries, OrderBook, Symbol,
        SymbolPrice, Transaction,
    },
};

pub struct Spot<'a> {
    client: &'a BinanceClient,
}

impl<'a> Spot<'a> {
    pub fn new(client: &'a BinanceClient) -> Self {
        Spot { client }
    }

    fn account(&self) -> Account {
        Account::new(self.client.api_key.clone(), self.client.secret_key.clone())
    }

    fn market(&self) -> Market {
        Market::new(self.client.api_key.clone(), self.client.secret_key.clone())
    }

    fn general(&self) -> General {
        General::new(self.client.api_key.clone(), self.client.secret_key.clone())
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
            .get_account()
            .map_err(|e| anyhow!(e.to_string()))?;

        Ok(account_information)
    }

    // 获取账户余额
    pub fn get_balance(&self, asset: impl Into<String>) -> Result<Balance> {
        let balance = self
            .account()
            .get_balance(asset)
            .map_err(|e| anyhow!(e.to_string()))?;

        Ok(balance)
    }

    // 限价买入
    pub fn limit_buy(
        &self,
        symbol: impl Into<String>, // 交易对
        qty: impl Into<f64>,       // 数量
        price: f64,                // 价格
    ) -> Result<Transaction> {
        let transaction = self
            .account()
            .limit_buy(symbol, qty, price)
            .map_err(|e| anyhow!(e.to_string()))?;

        Ok(transaction)
    }

    // 限价卖出
    pub fn limit_sell(
        &self,
        symbol: impl Into<String>,
        qty: impl Into<f64>, // 数量
        price: f64,          // 价格
    ) -> Result<Transaction> {
        let transaction = self
            .account()
            .limit_sell(symbol, qty, price)
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
