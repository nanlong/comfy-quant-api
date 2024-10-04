use anyhow::Result;
use bon::Builder;

#[derive(Builder)]
pub struct AccountInformation {
    pub maker_commission: f32,
    pub taker_commission: f32,
}

#[derive(Builder, Clone)]
#[builder(on(String, into))]
pub struct Balance {
    pub asset: String,  // 币种
    pub free: String,   // 可用余额
    pub locked: String, // 锁定余额
}

#[allow(unused)]
#[derive(Clone)]
pub enum OrderStatus {
    New,             // 新订单
    PartiallyFilled, // 部分成交
    Filled,          // 完全成交
    Canceled,        // 已撤销
    PendingCancel,   // 等待撤销
    Rejected,        // 已拒绝
}

#[derive(Clone)]
pub enum OrderType {
    Market,
    Limit,
}

#[derive(Clone)]
pub enum OrderSide {
    Buy,
    Sell,
}

#[allow(unused)]
#[derive(Builder, Clone)]
#[builder(on(String, into))]
pub struct Order {
    pub symbol: String,            // 交易对
    pub order_id: String,          // 订单ID
    pub price: String,             // 订单价格
    pub orig_qty: String,          // 用户设置的原始订单数量
    pub executed_qty: String,      // 已执行数量
    pub order_type: OrderType,     // 订单类型
    pub order_side: OrderSide,     // 订单方向
    pub order_status: OrderStatus, // 订单状态
    pub time: i64,                 // 创建时间
    pub update_time: i64,          // 更新时间
}

// 现货交易客户端
pub trait SpotOrderClient {
    // 获取账户信息，手续费
    fn get_account(&self) -> Result<AccountInformation>;

    // 获取账户余额
    fn get_balance(&self, asset: &str) -> Result<Balance>;

    // 获取订单信息
    fn get_order(&self, order_id: &str) -> Result<Order>;

    // 市价买单
    fn market_buy(&self, symbol: &str, qty: f64) -> Result<Order>;

    // 市价卖单
    fn market_sell(&self, symbol: &str, qty: f64) -> Result<Order>;

    // 限价买单
    fn limit_buy(&self, symbol: &str, qty: f64, price: f64) -> Result<Order>;

    // 限价卖单
    fn limit_sell(&self, symbol: &str, qty: f64, price: f64) -> Result<Order>;
}
