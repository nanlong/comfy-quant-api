use super::base_stats_data::BaseStatsData;
use comfy_quant_base::FuturesMarket;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

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
    pub market: FuturesMarket,            // 合约类型
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
