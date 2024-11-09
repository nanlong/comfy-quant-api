use std::str::FromStr;

use anyhow::{anyhow, Result};
use binance::model::{
    AccountInformation as BinanceAccountInformation, Balance as BinaceBalance,
    Order as BinanceOrder, Symbol as BinaceSymbolInformation, Transaction as BinanceTransaction,
};
use bon::Builder;
use rust_decimal::{prelude::FromPrimitive, Decimal};
use rust_decimal_macros::dec;

#[derive(Builder, Debug)]
pub struct AccountInformation {
    pub maker_commission_rate: Decimal,
    pub taker_commission_rate: Decimal,
    pub can_trade: bool,
}

impl TryFrom<BinanceAccountInformation> for AccountInformation {
    type Error = anyhow::Error;

    fn try_from(value: BinanceAccountInformation) -> std::result::Result<Self, Self::Error> {
        let maker_commission = Decimal::from_f32(value.maker_commission)
            .ok_or_else(|| anyhow!("binance account maker commission convert decimal failed"))?;
        let taker_commission = Decimal::from_f32(value.taker_commission)
            .ok_or_else(|| anyhow!("binance account taker commission convert decimal failed"))?;
        let maker_commission_rate = maker_commission / dec!(10000);
        let taker_commission_rate = taker_commission / dec!(10000);

        Ok(AccountInformation::builder()
            .maker_commission_rate(maker_commission_rate)
            .taker_commission_rate(taker_commission_rate)
            .can_trade(value.can_trade)
            .build())
    }
}

#[derive(Builder, Debug)]
pub struct SymbolInformation {
    pub symbol: String,
    pub base_asset: String,
    pub quote_asset: String,
    pub base_asset_precision: u32,
    pub quote_asset_precision: u32,
}

impl From<BinaceSymbolInformation> for SymbolInformation {
    fn from(value: BinaceSymbolInformation) -> Self {
        SymbolInformation::builder()
            .symbol(value.symbol)
            .base_asset(value.base_asset)
            .quote_asset(value.quote_asset)
            .base_asset_precision(value.base_asset_precision as u32)
            .quote_asset_precision(value.quote_precision as u32)
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

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "new" => Ok(OrderStatus::New),
            "partially_filled" => Ok(OrderStatus::PartiallyFilled),
            "filled" => Ok(OrderStatus::Filled),
            "canceled" => Ok(OrderStatus::Canceled),
            "pending_cancel" => Ok(OrderStatus::PendingCancel),
            "rejected" => Ok(OrderStatus::Rejected),
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

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "market" => Ok(OrderType::Market),
            "limit" => Ok(OrderType::Limit),
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

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "buy" => Ok(OrderSide::Buy),
            "sell" => Ok(OrderSide::Sell),
            _ => anyhow::bail!("OrderSize parse failed. value: {}", s),
        }
    }
}

#[derive(Builder, Debug, Clone)]
#[builder(on(String, into))]
pub struct Order {
    pub symbol: String,                  // 交易对
    pub order_id: String,                // 订单ID
    pub client_order_id: Option<String>, // 用户自己设置的ID
    pub price: String,                   // 订单价格
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
    pub fn base_asset_amount(&self) -> Result<Decimal> {
        Ok(self.executed_qty.parse::<Decimal>()?)
    }

    pub fn quote_asset_amount(&self) -> Result<Decimal> {
        Ok(self.cumulative_quote_qty.parse::<Decimal>()?)
    }
}

impl TryFrom<BinanceOrder> for Order {
    type Error = anyhow::Error;

    fn try_from(value: BinanceOrder) -> std::result::Result<Self, Self::Error> {
        let order_type = value.type_name.parse::<OrderType>()?;
        let order_side = value.side.parse::<OrderSide>()?;
        let order_status = value.status.parse::<OrderStatus>()?;

        let order = Order::builder()
            .symbol(value.symbol)
            .order_id(value.order_id.to_string())
            .client_order_id(value.client_order_id)
            .price(value.price.to_string())
            .orig_qty(value.orig_qty)
            .executed_qty(value.executed_qty)
            .cumulative_quote_qty(value.cummulative_quote_qty)
            .order_type(order_type)
            .order_side(order_side)
            .order_status(order_status)
            .time(value.time as i64)
            .update_time(value.update_time as i64)
            .build();

        Ok(order)
    }
}

impl TryFrom<BinanceTransaction> for Order {
    type Error = anyhow::Error;

    fn try_from(value: BinanceTransaction) -> std::result::Result<Self, Self::Error> {
        let order_type = value.type_name.parse::<OrderType>()?;
        let order_side = value.side.parse::<OrderSide>()?;
        let order_status = value.status.parse::<OrderStatus>()?;

        let order = Order::builder()
            .symbol(value.symbol)
            .order_id(value.order_id.to_string())
            .client_order_id(value.client_order_id)
            .price(value.price.to_string())
            .orig_qty(value.orig_qty.to_string())
            .executed_qty(value.executed_qty.to_string())
            .cumulative_quote_qty(value.cummulative_quote_qty.to_string())
            .order_type(order_type)
            .order_side(order_side)
            .order_status(order_status)
            .time(0)
            .update_time(0)
            .build();

        Ok(order)
    }
}
