// use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

// 基础统计数据结构
#[derive(Serialize, Deserialize, Debug)]
pub struct BaseStatsData {
    pub exchange: String,                // 交易所
    pub symbol: String,                  // 交易对
    pub base_asset: String,              // 基础资产
    pub quote_asset: String,             // 报价资产
    pub total_trades: u64,               // 总交易次数
    pub buy_trades: u64,                 // 买入交易次数
    pub sell_trades: u64,                // 卖出交易次数
    pub win_trades: u64,                 // 盈利交易次数
    pub maker_commission_rate: Decimal,  // 挂单手续费率
    pub taker_commission_rate: Decimal,  // 吃单手续费率
    pub total_base_commission: Decimal,  // 总基础资产手续费
    pub total_quote_commission: Decimal, // 总报价资产手续费
    pub total_base_volume: Decimal,      // 总基础资产交易量
    pub total_quote_volume: Decimal,     // 总报价资产交易量
    pub realized_pnl: Decimal,           // 已实现盈亏
    pub unrealized_pnl: Decimal,         // 未实现盈亏
}

// 现货统计
#[derive(Serialize, Deserialize, Debug)]
pub struct SpotStatsData {
    #[serde(flatten)]
    base: BaseStatsData,

    // 现货特有字段
    pub initial_base_balance: Decimal,  // 初始基础资产余额
    pub initial_quote_balance: Decimal, // 初始报价资产余额
    pub initial_price: Decimal,         // 初始价格
    pub base_asset_balance: Decimal,    // 基础资产余额
    pub quote_asset_balance: Decimal,   // 报价资产余额
    pub avg_price: Decimal,             // 平均价格
}

// 合约类型
#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub enum FuturesType {
    UsdtSettled, // U本位合约
    CoinSettled, // 币本位合约
}

// 持仓方向
#[derive(Serialize, Deserialize, Debug)]
pub enum PositionSide {
    Long,
    Short,
}

// 合约统计
#[derive(Serialize, Deserialize, Debug)]
pub struct FuturesStatsData {
    #[serde(flatten)]
    base: BaseStatsData,

    pub futures_type: FuturesType,        // 合约类型
    pub contract_size: Decimal,           // 合约面值
    pub leverage: Decimal,                // 杠杆倍数
    pub margin_asset: String,             // 保证金资产(USDT或BTC等)
    pub margin_balance: Decimal,          // 保证金余额
    pub margin_ratio: Decimal,            // 保证金率
    pub maintenance_margin_rate: Decimal, // 维持保证金率
    pub position_size: Decimal,           // 持仓量(正数为多,负数为空)
    pub position_value: Decimal,          // 仓位价值(以保证金资产计价)
    pub entry_price: Decimal,             // 开仓均价
    pub mark_price: Decimal,              // 标记价格
    pub liquidation_price: Decimal,       // 强平价格
    pub funding_fee: Decimal,             // 资金费用(以保证金资产计价)
}

// // 基础交易统计trait
// pub trait TradeStats {
//     // 总盈亏
//     fn total_pnl(&self) -> Decimal;
//     // 未实现盈亏
//     fn unrealized_pnl(&self, current_price: &Decimal) -> Decimal;
//     // 已实现盈亏
//     fn realized_pnl(&self) -> Decimal;
//     // ... 其他共同方法
// }

// // 实现特定的统计逻辑
// impl TradeStats for SpotStatsData {
//     // 实现现货的PNL计算等
// }

// impl TradeStats for FuturesStatsData {
//     // 实现合约的PNL计算等
// }
