use super::base_stats_data::BaseStatsData;
use crate::node_core::{NodeContext, Tick};
use anyhow::Result;
use chrono::Utc;
use comfy_quant_base::{Exchange, Symbol};
use comfy_quant_database::{
    strategy_spot_position::{self, StrategySpotPosition},
    strategy_spot_stats::{self, StrategySpotStats},
    SpotStatsQuery,
};
use comfy_quant_exchange::client::spot_client::base::{Order, OrderSide};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

// 现货统计
#[derive(Serialize, Deserialize, Debug, Default)]
pub struct SpotStatsData {
    #[serde(flatten)]
    pub base: BaseStatsData,

    // 现货特有字段
    pub initial_base_balance: Decimal,  // 初始基础资产余额
    pub initial_quote_balance: Decimal, // 初始报价资产余额
    pub initial_price: Decimal,         // 初始价格
    pub base_asset_balance: Decimal,    // 基础资产余额
    pub quote_asset_balance: Decimal,   // 报价资产余额
    pub avg_price: Decimal,             // 平均价格
}

#[allow(unused)]
impl SpotStatsData {
    pub fn new() -> Self {
        SpotStatsData::default()
    }

    pub fn setup(
        &mut self,
        exchange: &Exchange,
        symbol: &Symbol,
        base_asset: &str,
        quote_asset: &str,
    ) {
        self.base.exchange = exchange.clone();
        self.base.symbol = symbol.clone();
        self.base.base_asset = base_asset.into();
        self.base.quote_asset = quote_asset.into();
    }

    fn params<'a>(&'a self, workflow_id: &'a str, node_id: i16) -> SpotStatsQuery<'a> {
        SpotStatsQuery::builder()
            .workflow_id(workflow_id)
            .node_id(node_id)
            .exchange(&self.base.exchange)
            .symbol(&self.base.symbol)
            .build()
    }

    pub async fn initialize_balance(
        &mut self,
        ctx: &NodeContext,
        initial_base: &Decimal,
        initial_quote: &Decimal,
        initial_price: &Decimal,
    ) -> Result<()> {
        self.initial_base_balance = initial_base.to_owned();
        self.initial_quote_balance = initial_quote.to_owned();
        self.initial_price = initial_price.to_owned();
        self.base_asset_balance = initial_base.to_owned();
        self.quote_asset_balance = initial_quote.to_owned();

        self.save_strategy_spot_stats(
            ctx.db(),
            ctx.node_name(),
            &self.base.base_asset,
            &self.base.quote_asset,
            &self.params(ctx.workflow_id(), ctx.node_id()),
        )
        .await?;

        Ok(())
    }

    pub async fn update_with_tick(&mut self, _ctx: &NodeContext, tick: &Tick) -> Result<()> {
        // 更新未实现盈亏
        self.base.unrealized_pnl = self.base_asset_balance * (tick.price - self.avg_price);

        Ok(())
    }

    pub async fn update_with_order(&mut self, ctx: &NodeContext, order: &Order) -> Result<()> {
        let now = Utc::now();
        let base_asset_amount = order.base_asset_amount()?;
        let quote_asset_amount = order.quote_asset_amount()?;
        let base_commission = order.base_commission(&self.base.maker_commission_rate)?;
        let quote_commission = order.quote_commission(&self.base.maker_commission_rate)?;
        let order_avg_price = order.avg_price.parse::<Decimal>()?;

        self.base.total_trades += 1;
        self.base.total_base_volume += base_asset_amount;
        self.base.total_quote_volume += quote_asset_amount;

        match order.order_side {
            OrderSide::Buy => {
                // 扣除手续费后实际获得
                let base_amount = base_asset_amount - base_commission;
                // 持仓均价
                let avg_price = (self.base_asset_balance * self.avg_price
                    + base_amount * order_avg_price)
                    / (self.base_asset_balance + base_amount);

                self.base.buy_trades += 1;
                self.base_asset_balance += base_amount;
                self.avg_price = avg_price;
                self.quote_asset_balance -= quote_asset_amount;
                self.base.total_base_commission += base_commission;
            }
            OrderSide::Sell => {
                // 扣除手续费后实际获得
                let quote_amount = quote_asset_amount - quote_commission;
                // 成本
                let cost = base_asset_amount * self.avg_price;

                self.base.sell_trades += 1;
                self.base_asset_balance -= base_asset_amount;
                self.quote_asset_balance += quote_amount;
                self.base.total_quote_commission += quote_commission;

                // 卖出所得大于成本，则确定为一次盈利交易
                if quote_amount > cost {
                    self.base.win_trades += 1;
                }

                // 已实现总盈亏
                self.base.realized_pnl += quote_amount - cost;
            }
        }

        let params = self.params(ctx.workflow_id(), ctx.node_id());

        self.save_strategy_spot_stats(
            ctx.db(),
            ctx.node_name(),
            &self.base.base_asset,
            &self.base.quote_asset,
            &params,
        )
        .await?;
        self.save_strategy_spot_position(
            ctx.db(),
            ctx.node_name(),
            &self.base.base_asset,
            &self.base.quote_asset,
            &params,
        )
        .await?;

        Ok(())
    }

    // 保存策略持仓
    pub async fn save_strategy_spot_position(
        &self,
        db: &PgPool,
        node_name: &str,
        base_asset: &str,
        quote_asset: &str,
        params: &SpotStatsQuery<'_>,
    ) -> Result<()> {
        let data = StrategySpotPosition::builder()
            .workflow_id(params.workflow_id)
            .node_id(params.node_id)
            .node_name(node_name)
            .exchange(params.exchange.clone())
            .symbol(params.symbol.clone())
            .base_asset(base_asset)
            .quote_asset(quote_asset)
            .base_asset_balance(self.base_asset_balance)
            .quote_asset_balance(self.quote_asset_balance)
            .realized_pnl(self.base.realized_pnl)
            .build();

        strategy_spot_position::create(db, &data).await?;

        Ok(())
    }

    // 保存策略统计数据
    pub async fn save_strategy_spot_stats(
        &self,
        db: &PgPool,
        node_name: &str,
        base_asset: &str,
        quote_asset: &str,
        params: &SpotStatsQuery<'_>,
    ) -> Result<()> {
        let data = StrategySpotStats::builder()
            .workflow_id(params.workflow_id)
            .node_id(params.node_id)
            .node_name(node_name)
            .exchange(params.exchange.clone())
            .symbol(params.symbol.clone())
            .base_asset(base_asset)
            .quote_asset(quote_asset)
            .initial_base_balance(self.initial_base_balance)
            .initial_quote_balance(self.initial_quote_balance)
            .initial_price(self.initial_price)
            .maker_commission_rate(self.base.maker_commission_rate)
            .taker_commission_rate(self.base.taker_commission_rate)
            .base_asset_balance(self.base_asset_balance)
            .quote_asset_balance(self.quote_asset_balance)
            .avg_price(self.avg_price)
            .total_trades(self.base.total_trades as i64)
            .buy_trades(self.base.buy_trades as i64)
            .sell_trades(self.base.sell_trades as i64)
            .total_base_volume(self.base.total_base_volume)
            .total_quote_volume(self.base.total_quote_volume)
            .total_base_commission(self.base.total_base_commission)
            .total_quote_commission(self.base.total_quote_commission)
            .realized_pnl(self.base.realized_pnl)
            .win_trades(self.base.win_trades as i64)
            .build();

        strategy_spot_stats::create_or_update(db, &data).await?;

        Ok(())
    }
}
