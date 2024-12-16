use comfy_quant_base::{Exchange, Symbol};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

// 基础统计数据结构
#[derive(Serialize, Deserialize, Debug, Default)]
pub struct BaseStatsData {
    pub exchange: Exchange,              // 交易所
    pub symbol: Symbol,                  // 交易对
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
