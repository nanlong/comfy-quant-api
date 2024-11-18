use anyhow::Result;
use comfy_quant_exchange::client::spot_client::base::{Order, OrderSide};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use sqlx::PgPool;

/// 节点统计数据
#[derive(Debug, Default, Clone)]
#[allow(unused)]
pub struct Stats {
    pub initial_base_balance: Decimal,   // 初始化base资产余额
    pub initial_quote_balance: Decimal,  // 初始化quote资产余额
    pub base_asset_balance: Decimal,     // base资产持仓量
    pub quote_asset_balance: Decimal,    // quote资产持仓量
    pub avg_price: Decimal,              // base资产持仓均价
    pub total_trades: u64,               // 总交易次数
    pub buy_trades: u64,                 // 买入次数
    pub sell_trades: u64,                // 卖出次数
    pub total_base_volume: Decimal,      // base资产交易量
    pub total_quote_volume: Decimal,     // quote资产交易量
    pub total_base_commission: Decimal,  // 总手续费
    pub total_quote_commission: Decimal, // 总手续费
    pub realized_pnl: Decimal,           // 已实现盈亏
    pub win_trades: u64,                 // 盈利交易次数
    pub max_drawdown: Decimal,           // 最大回撤
    pub roi: Decimal,                    // 收益率
}

#[allow(unused)]
impl Stats {
    pub fn update_with_order(
        &mut self,
        _db: &PgPool,
        order: &Order,
        commission_rate: &Decimal,
    ) -> Result<()> {
        let base_asset_amount = order.base_asset_amount()?;
        let quote_asset_amount = order.quote_asset_amount()?;
        let base_commission = order.base_commission(commission_rate)?;
        let quote_commission = order.quote_commission(commission_rate)?;
        let order_avg_price = order.avg_price.parse::<Decimal>()?;

        self.total_trades += 1;
        self.total_base_volume += base_asset_amount;
        self.total_quote_volume += quote_asset_amount;

        match order.order_side {
            OrderSide::Buy => {
                // 扣除手续费后实际获得
                let base_amount = base_asset_amount - base_commission;
                // 持仓均价
                let avg_price = (self.base_asset_balance * self.avg_price
                    + base_amount * order_avg_price)
                    / (self.base_asset_balance + base_amount);

                self.buy_trades += 1;
                self.base_asset_balance += base_amount;
                self.avg_price = avg_price;
                self.quote_asset_balance -= quote_asset_amount;
                self.total_base_commission += base_commission;
            }
            OrderSide::Sell => {
                // 扣除手续费后实际获得
                let quote_amount = quote_asset_amount - quote_commission;
                // 成本
                let cost = base_asset_amount * self.avg_price;

                self.sell_trades += 1;
                self.base_asset_balance -= base_asset_amount;
                self.quote_asset_balance += quote_amount;
                self.total_quote_commission += quote_commission;

                // 卖出所得大于成本，则确定为一次盈利交易
                if quote_amount > cost {
                    self.win_trades += 1;
                }

                // 已实现总盈亏
                self.realized_pnl += quote_amount - cost;
            }
        }

        Ok(())
    }

    // 已实现盈亏
    pub fn realized_pnl(&self) -> Decimal {
        self.realized_pnl
    }

    // 未实现盈亏
    pub fn unrealized_pnl(&self, price: &Decimal, commission_rate: &Decimal) -> Decimal {
        let cost = self.base_asset_balance * self.avg_price;
        let maybe_sell = self.base_asset_balance * price * (dec!(1) - commission_rate);
        maybe_sell - cost
    }
}
