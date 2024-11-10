use anyhow::{anyhow, Result};
use binance::model::{
    AccountInformation as BinanceAccountInformation, Balance as BinaceBalance,
    Filters as BinanceFilters, Order as BinanceOrder, Symbol as BinaceSymbolInformation,
    SymbolPrice as BinanceSymbolPrice, Transaction as BinanceTransaction,
};
use bon::Builder;
use rust_decimal::{prelude::FromPrimitive, Decimal};
use rust_decimal_macros::dec;
use std::str::FromStr;

#[derive(Builder, Debug)]
pub struct AccountInformation {
    pub maker_commission_rate: Decimal,
    pub taker_commission_rate: Decimal,
    pub can_trade: bool,
}

impl TryFrom<BinanceAccountInformation> for AccountInformation {
    type Error = anyhow::Error;

    fn try_from(value: BinanceAccountInformation) -> std::result::Result<Self, Self::Error> {
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
            BinanceFilters::Notional { min_notional, .. } => min_notional
                .as_ref()
                .and_then(|val| val.parse::<Decimal>().ok()),
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

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
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

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
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

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "BUY" => Ok(OrderSide::Buy),
            "SELL" => Ok(OrderSide::Sell),
            _ => anyhow::bail!("OrderSide parse failed. value: {}", s),
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

        let amount = value.cummulative_quote_qty.parse::<Decimal>()?;
        let qty = value.executed_qty.parse::<Decimal>()?;
        let avg_price = amount / qty;

        let order = Order::builder()
            .symbol(value.symbol)
            .order_id(value.order_id.to_string())
            .client_order_id(value.client_order_id)
            .price(value.price.to_string())
            .avg_price(avg_price.to_string())
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

        let amount = Decimal::from_f64(value.cummulative_quote_qty).ok_or_else(|| {
            anyhow!("binance transaction cummulative quote qty convert decimal failed")
        })?;
        let qty = Decimal::from_f64(value.executed_qty)
            .ok_or_else(|| anyhow!("binance transaction executed qty convert decimal failed"))?;
        let avg_price = amount / qty;

        let order = Order::builder()
            .symbol(value.symbol)
            .order_id(value.order_id.to_string())
            .client_order_id(value.client_order_id)
            .price(value.price.to_string())
            .avg_price(avg_price.to_string())
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

#[derive(Builder, Debug, Clone)]
#[builder(on(String, into))]
pub struct SymbolPrice {
    pub symbol: String,
    pub price: Decimal,
}

impl TryFrom<BinanceSymbolPrice> for SymbolPrice {
    type Error = anyhow::Error;

    fn try_from(value: BinanceSymbolPrice) -> std::result::Result<Self, Self::Error> {
        let price = Decimal::from_f64(value.price)
            .ok_or_else(|| anyhow!("binance symbol price convert decimal failed"))?;

        Ok(SymbolPrice::builder()
            .symbol(value.symbol)
            .price(price)
            .build())
    }
}
