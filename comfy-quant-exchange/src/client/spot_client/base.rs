use anyhow::Result;
use bon::Builder;
use rust_decimal::Decimal;

#[derive(Builder, Debug)]
pub struct AccountInformation {
    pub maker_commission: f64,
    pub taker_commission: f64,
}

#[derive(Builder, Debug)]
pub struct SymbolInformation {
    pub symbol: String,
    pub base_asset: String,
    pub quote_asset: String,
    pub base_asset_precision: u32,
    pub quote_asset_precision: u32,
}

#[derive(Builder, Debug, Clone)]
#[builder(on(String, into))]
pub struct Balance {
    pub asset: String,  // 币种
    pub free: String,   // 可用余额
    pub locked: String, // 锁定余额
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

#[derive(Debug, Clone)]
pub enum OrderType {
    Market,
    Limit,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OrderSide {
    Buy,
    Sell,
}

#[derive(Builder, Debug, Clone)]
#[builder(on(String, into))]
pub struct Order {
    pub symbol: String,       // 交易对
    pub id: String,           // 订单ID
    pub price: String,        // 订单价格
    pub orig_qty: String,     // 用户设置的原始订单数量
    pub executed_qty: String, // 已执行数量
    pub r#type: OrderType,    // 订单类型
    pub side: OrderSide,      // 订单方向
    pub status: OrderStatus,  // 订单状态
    pub time: i64,            // 创建时间
    pub update_time: i64,     // 更新时间
}

impl Order {
    pub fn base_asset_amount(&self) -> Result<Decimal> {
        Ok(self.executed_qty.parse::<Decimal>()?)
    }

    pub fn quote_asset_amount(&self) -> Result<Decimal> {
        let base_asset_amount = self.base_asset_amount()?;
        let price = self.price.parse::<Decimal>()?;
        Ok(base_asset_amount * price)
    }
}
