use anyhow::anyhow;
use binance::model::{
    AccountInformation as BinanceAccountInformation, Balance as BinaceBalance,
    Filters as BinanceFilters, Symbol as BinaceSymbolInformation,
    SymbolPrice as BinanceSymbolPrice,
};
use bon::Builder;
use rust_decimal::{prelude::FromPrimitive, Decimal};
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

pub const BACKTEST_EXCHANGE_NAME: &str = "Backtest";
pub const BINANCE_EXCHANGE_NAME: &str = "Binance";

#[derive(Builder)]
#[builder(on(String, into))]
pub struct BinanceOrder {
    base_asset: String,
    quote_asset: String,
    order: binance::model::Order,
}

#[derive(Builder)]
#[builder(on(String, into))]
pub struct BinanceTransaction {
    base_asset: String,
    quote_asset: String,
    transaction: binance::model::Transaction,
}

#[derive(Builder, Debug, Clone, Default)]
pub struct AccountInformation {
    pub maker_commission_rate: Decimal,
    pub taker_commission_rate: Decimal,
    pub can_trade: bool,
}

impl TryFrom<BinanceAccountInformation> for AccountInformation {
    type Error = anyhow::Error;

    fn try_from(value: BinanceAccountInformation) -> Result<Self, Self::Error> {
        let to_rate = |val: f32, commission_type: &str| {
            Decimal::from_f32(val)
                .ok_or_else(|| {
                    anyhow!(
                        "binance account {} commission convert decimal failed",
                        commission_type
                    )
                })
                .map(|v| v / dec!(10000))
        };

        Ok(AccountInformation::builder()
            .maker_commission_rate(to_rate(value.maker_commission, "maker")?)
            .taker_commission_rate(to_rate(value.taker_commission, "taker")?)
            .can_trade(value.can_trade)
            .build())
    }
}

#[derive(Builder, Debug)]
#[builder(on(String, into))]
pub struct SymbolInformation {
    pub symbol: String,
    pub base_asset: String,
    pub quote_asset: String,
    pub base_asset_precision: u32,
    pub quote_asset_precision: u32,
    pub min_notional: Option<Decimal>,
}

impl From<BinaceSymbolInformation> for SymbolInformation {
    fn from(value: BinaceSymbolInformation) -> Self {
        let min_notional = value.filters.iter().find_map(|filter| match filter {
            BinanceFilters::Notional { min_notional, .. } => {
                min_notional.as_ref().and_then(|val| val.parse().ok())
            }
            _ => None,
        });

        SymbolInformation::builder()
            .symbol(value.symbol)
            .base_asset(value.base_asset)
            .quote_asset(value.quote_asset)
            .base_asset_precision(value.base_asset_precision as u32)
            .quote_asset_precision(value.quote_precision as u32)
            .maybe_min_notional(min_notional)
            .build()
    }
}

#[derive(Builder, Debug, Clone)]
#[builder(on(String, into))]
pub struct Balance {
    pub asset: String,  // 币种
    pub free: String,   // 可用余额
    pub locked: String, // 锁定余额
}

impl From<BinaceBalance> for Balance {
    fn from(value: BinaceBalance) -> Self {
        Balance::builder()
            .asset(value.asset)
            .free(value.free)
            .locked(value.locked)
            .build()
    }
}

#[derive(Debug, Clone)]
pub enum OrderStatus {
    New,             // 新订单
    PartiallyFilled, // 部分成交
    Filled,          // 完全成交
    Canceled,        // 已撤销
    PendingCancel,   // 等待撤销
    Rejected,        // 已拒绝
}

impl FromStr for OrderStatus {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "NEW" => Ok(OrderStatus::New),
            "PARTIALLY_FILLED" => Ok(OrderStatus::PartiallyFilled),
            "FILLED" => Ok(OrderStatus::Filled),
            "CANCELED" => Ok(OrderStatus::Canceled),
            "PENDING_CANCEL" => Ok(OrderStatus::PendingCancel),
            "REJECTED" => Ok(OrderStatus::Rejected),
            _ => anyhow::bail!("OrderStatus parse failed. value: {}", s),
        }
    }
}

#[derive(Debug, Clone)]
pub enum OrderType {
    Market,
    Limit,
}

impl FromStr for OrderType {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "MARKET" => Ok(OrderType::Market),
            "LIMIT" => Ok(OrderType::Limit),
            _ => anyhow::bail!("OrderType parse failed. value: {}", s),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OrderSide {
    Buy,
    Sell,
}

impl FromStr for OrderSide {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "BUY" => Ok(OrderSide::Buy),
            "SELL" => Ok(OrderSide::Sell),
            _ => anyhow::bail!("OrderSide parse failed. value: {}", s),
        }
    }
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Hash, Debug, Clone)]
pub struct Exchange(String);

impl Exchange {
    fn new(s: impl Into<String>) -> Self {
        Exchange(s.into())
    }
}

impl From<String> for Exchange {
    fn from(value: String) -> Self {
        Exchange::new(value)
    }
}

impl From<&str> for Exchange {
    fn from(value: &str) -> Self {
        Exchange::new(value)
    }
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Hash, Debug, Clone)]
pub struct Symbol(String);

impl Symbol {
    fn new(s: impl Into<String>) -> Self {
        Symbol(s.into())
    }
}

impl From<String> for Symbol {
    fn from(value: String) -> Self {
        Symbol::new(value)
    }
}

impl From<&str> for Symbol {
    fn from(value: &str) -> Self {
        Symbol::new(value)
    }
}

#[derive(Builder, Debug, Clone)]
#[builder(on(String, into), on(Exchange, into), on(Symbol, into))]
#[allow(clippy::duplicated_attributes)]
pub struct Order {
    pub exchange: Exchange,              // 交易所
    pub base_asset: Option<String>,      // 基础货币
    pub quote_asset: Option<String>,     // 计价货币
    pub symbol: Symbol,                  // 交易对
    pub order_id: String,                // 订单ID
    pub client_order_id: Option<String>, // 用户自己设置的ID
    pub price: String,                   // 订单价格
    pub avg_price: String,               // 平均成交价格
    pub orig_qty: String,                // 用户设置的原始订单数量
    pub executed_qty: String,            // 交易的订单数量
    pub cumulative_quote_qty: String,    // 累计交易的金额
    pub order_type: OrderType,           // 订单类型
    pub order_side: OrderSide,           // 订单方向
    pub order_status: OrderStatus,       // 订单状态
    pub time: i64,                       // 订单时间
    pub update_time: i64,                // 最后更新时间
}

impl Order {
    pub fn base_asset(&self) -> anyhow::Result<&String> {
        self.base_asset
            .as_ref()
            .ok_or_else(|| anyhow!("base asset not set"))
    }

    pub fn quote_asset(&self) -> anyhow::Result<&String> {
        self.quote_asset
            .as_ref()
            .ok_or_else(|| anyhow!("quote asset not set"))
    }

    pub fn base_asset_amount(&self) -> anyhow::Result<Decimal> {
        Ok(self.executed_qty.parse()?)
    }

    pub fn quote_asset_amount(&self) -> anyhow::Result<Decimal> {
        Ok(self.cumulative_quote_qty.parse()?)
    }

    pub fn base_commission(&self, commission_rate: &Decimal) -> anyhow::Result<Decimal> {
        let commission = match self.order_side {
            OrderSide::Buy => self.base_asset_amount()? * commission_rate,
            OrderSide::Sell => dec!(0),
        };

        Ok(commission)
    }

    pub fn quote_commission(&self, commission_rate: &Decimal) -> anyhow::Result<Decimal> {
        let commission = match self.order_side {
            OrderSide::Buy => dec!(0),
            OrderSide::Sell => self.quote_asset_amount()? * commission_rate,
        };

        Ok(commission)
    }
}

impl TryFrom<BinanceOrder> for Order {
    type Error = anyhow::Error;

    fn try_from(value: BinanceOrder) -> Result<Self, Self::Error> {
        let order_type = value.order.type_name.parse::<OrderType>()?;
        let order_side = value.order.side.parse::<OrderSide>()?;
        let order_status = value.order.status.parse::<OrderStatus>()?;

        let amount = value.order.cummulative_quote_qty.parse::<Decimal>()?;
        let qty = value.order.executed_qty.parse::<Decimal>()?;
        let avg_price = amount / qty;

        let order = Order::builder()
            .exchange(BINANCE_EXCHANGE_NAME)
            .base_asset(value.base_asset)
            .quote_asset(value.quote_asset)
            .symbol(value.order.symbol)
            .order_id(value.order.order_id.to_string())
            .client_order_id(value.order.client_order_id)
            .price(value.order.price.to_string())
            .avg_price(avg_price.to_string())
            .orig_qty(value.order.orig_qty)
            .executed_qty(value.order.executed_qty)
            .cumulative_quote_qty(value.order.cummulative_quote_qty)
            .order_type(order_type)
            .order_side(order_side)
            .order_status(order_status)
            .time(value.order.time as i64)
            .update_time(value.order.update_time as i64)
            .build();

        Ok(order)
    }
}

impl TryFrom<BinanceTransaction> for Order {
    type Error = anyhow::Error;

    fn try_from(value: BinanceTransaction) -> Result<Self, Self::Error> {
        let order_type = value.transaction.type_name.parse::<OrderType>()?;
        let order_side = value.transaction.side.parse::<OrderSide>()?;
        let order_status = value.transaction.status.parse::<OrderStatus>()?;

        let amount =
            Decimal::from_f64(value.transaction.cummulative_quote_qty).ok_or_else(|| {
                anyhow!("binance transaction cummulative quote qty convert decimal failed")
            })?;
        let qty = Decimal::from_f64(value.transaction.executed_qty)
            .ok_or_else(|| anyhow!("binance transaction executed qty convert decimal failed"))?;
        let avg_price = amount / qty;

        let order = Order::builder()
            .exchange(BINANCE_EXCHANGE_NAME)
            .base_asset(value.base_asset)
            .quote_asset(value.quote_asset)
            .symbol(value.transaction.symbol)
            .order_id(value.transaction.order_id.to_string())
            .client_order_id(value.transaction.client_order_id)
            .price(value.transaction.price.to_string())
            .avg_price(avg_price.to_string())
            .orig_qty(value.transaction.orig_qty.to_string())
            .executed_qty(value.transaction.executed_qty.to_string())
            .cumulative_quote_qty(value.transaction.cummulative_quote_qty.to_string())
            .order_type(order_type)
            .order_side(order_side)
            .order_status(order_status)
            .time(0)
            .update_time(0)
            .build();

        Ok(order)
    }
}

#[derive(Builder, Debug, Clone, PartialEq, Eq)]
#[builder(on(String, into))]
pub struct SymbolPrice {
    pub symbol: String,
    pub price: Decimal,
}

impl TryFrom<BinanceSymbolPrice> for SymbolPrice {
    type Error = anyhow::Error;

    fn try_from(value: BinanceSymbolPrice) -> Result<Self, Self::Error> {
        let price = Decimal::from_f64(value.price)
            .ok_or_else(|| anyhow!("binance symbol price convert decimal failed"))?;

        Ok(SymbolPrice::builder()
            .symbol(value.symbol)
            .price(price)
            .build())
    }
}

#[derive(Clone)]
pub enum SpotClientRequest {
    Exchange,
    Symbol {
        base_asset: String,
        quote_asset: String,
    },
    GetAccount,
    GetSymbolInfo {
        base_asset: String,
        quote_asset: String,
    },
    GetBalance {
        asset: String,
    },
    GetOrder {
        base_asset: String,
        quote_asset: String,
        order_id: String,
    },
    MarketBuy {
        base_asset: String,
        quote_asset: String,
        qty: f64,
    },
    MarketSell {
        base_asset: String,
        quote_asset: String,
        qty: f64,
    },
    LimitBuy {
        base_asset: String,
        quote_asset: String,
        qty: f64,
        price: f64,
    },
    LimitSell {
        base_asset: String,
        quote_asset: String,
        qty: f64,
        price: f64,
    },
    GetPrice {
        base_asset: String,
        quote_asset: String,
    },
}

impl SpotClientRequest {
    pub fn exchange() -> Self {
        SpotClientRequest::Exchange
    }

    pub fn get_account() -> Self {
        SpotClientRequest::GetAccount
    }

    pub fn get_symbol_info(base_asset: impl Into<String>, quote_asset: impl Into<String>) -> Self {
        SpotClientRequest::GetSymbolInfo {
            base_asset: base_asset.into(),
            quote_asset: quote_asset.into(),
        }
    }

    pub fn get_balance(asset: impl Into<String>) -> Self {
        SpotClientRequest::GetBalance {
            asset: asset.into(),
        }
    }
}

pub enum SpotClientResponse {
    Exchange(String),
    Symbol(String),
    AccountInformation(AccountInformation),
    SymbolInformation(SymbolInformation),
    Balance(Balance),
    Order(Order),
    SymbolPrice(SymbolPrice),
}

impl From<String> for SpotClientResponse {
    fn from(value: String) -> Self {
        SpotClientResponse::Exchange(value)
    }
}

impl From<AccountInformation> for SpotClientResponse {
    fn from(value: AccountInformation) -> Self {
        SpotClientResponse::AccountInformation(value)
    }
}

impl From<SymbolInformation> for SpotClientResponse {
    fn from(value: SymbolInformation) -> Self {
        SpotClientResponse::SymbolInformation(value)
    }
}

impl From<Balance> for SpotClientResponse {
    fn from(value: Balance) -> Self {
        SpotClientResponse::Balance(value)
    }
}

impl From<Order> for SpotClientResponse {
    fn from(value: Order) -> Self {
        SpotClientResponse::Order(value)
    }
}

impl From<SymbolPrice> for SpotClientResponse {
    fn from(value: SymbolPrice) -> Self {
        SpotClientResponse::SymbolPrice(value)
    }
}

impl TryFrom<SpotClientResponse> for String {
    type Error = anyhow::Error;

    fn try_from(value: SpotClientResponse) -> Result<Self, Self::Error> {
        let SpotClientResponse::Exchange(exchange) = value else {
            anyhow::bail!("try from SpotClientResponse to String failed")
        };

        Ok(exchange)
    }
}

impl TryFrom<SpotClientResponse> for AccountInformation {
    type Error = anyhow::Error;

    fn try_from(value: SpotClientResponse) -> Result<Self, Self::Error> {
        let SpotClientResponse::AccountInformation(account_information) = value else {
            anyhow::bail!("try from SpotClientResponse to AccountInformation failed")
        };

        Ok(account_information)
    }
}

impl TryFrom<SpotClientResponse> for SymbolInformation {
    type Error = anyhow::Error;

    fn try_from(value: SpotClientResponse) -> Result<Self, Self::Error> {
        let SpotClientResponse::SymbolInformation(symbol_information) = value else {
            anyhow::bail!("try from SpotClientResponse to SymbolInformation failed")
        };

        Ok(symbol_information)
    }
}

impl TryFrom<SpotClientResponse> for Balance {
    type Error = anyhow::Error;

    fn try_from(value: SpotClientResponse) -> Result<Self, Self::Error> {
        let SpotClientResponse::Balance(balance) = value else {
            anyhow::bail!("try from SpotClientResponse to Balance failed")
        };

        Ok(balance)
    }
}

impl TryFrom<SpotClientResponse> for Order {
    type Error = anyhow::Error;

    fn try_from(value: SpotClientResponse) -> Result<Self, Self::Error> {
        let SpotClientResponse::Order(order) = value else {
            anyhow::bail!("try from SpotClientResponse to Order failed")
        };

        Ok(order)
    }
}

impl TryFrom<SpotClientResponse> for SymbolPrice {
    type Error = anyhow::Error;

    fn try_from(value: SpotClientResponse) -> Result<Self, Self::Error> {
        let SpotClientResponse::SymbolPrice(symbol_price) = value else {
            anyhow::bail!("try from SpotClientResponse to SymbolPrice failed")
        };

        Ok(symbol_price)
    }
}
