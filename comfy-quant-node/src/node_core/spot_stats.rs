use super::NodeContext;
use anyhow::Result;
use chrono::{DateTime, Utc};
use comfy_quant_base::ExchangeSymbolKey;
use comfy_quant_database::{
    kline::Kline,
    strategy_spot_position::{self, StrategySpotPosition},
    strategy_spot_stats::{self, StrategySpotStats},
    SpotStatsQuery,
};
use comfy_quant_exchange::client::spot_client::base::{Order, OrderSide};
use polars::{
    df,
    prelude::{
        col, lit, DataFrameJoinOps, FillNullStrategy, IntoLazy, JoinArgs, JoinType,
        SortMultipleOptions,
    },
};
use rust_decimal::{prelude::ToPrimitive, Decimal, MathematicalOps};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::collections::HashMap;

type SpotStatsDataMap = HashMap<ExchangeSymbolKey, SpotStatsData>;

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct SpotStats {
    data: SpotStatsDataMap,
}

impl AsRef<SpotStatsDataMap> for SpotStats {
    fn as_ref(&self) -> &SpotStatsDataMap {
        &self.data
    }
}

impl AsMut<SpotStatsDataMap> for SpotStats {
    fn as_mut(&mut self) -> &mut SpotStatsDataMap {
        &mut self.data
    }
}

impl SpotStats {
    pub fn new() -> Self {
        SpotStats {
            data: SpotStatsDataMap::new(),
        }
    }

    pub fn get(
        &self,
        exchange: impl AsRef<str>,
        symbol: impl AsRef<str>,
    ) -> Option<&SpotStatsData> {
        let key = ExchangeSymbolKey::new(exchange.as_ref(), symbol.as_ref());
        self.data.get(&key)
    }

    pub fn get_or_insert(
        &mut self,
        exchange: impl AsRef<str>,
        symbol: impl AsRef<str>,
    ) -> &mut SpotStatsData {
        let key = ExchangeSymbolKey::new(exchange.as_ref(), symbol.as_ref());
        self.as_mut().entry(key).or_default()
    }

    pub fn setup(
        &mut self,
        exchange: impl AsRef<str>,
        symbol: impl AsRef<str>,
        base_asset: impl AsRef<str>,
        quote_asset: impl AsRef<str>,
    ) {
        self.get_or_insert(exchange.as_ref(), symbol.as_ref())
            .setup(
                exchange.as_ref(),
                symbol.as_ref(),
                base_asset.as_ref(),
                quote_asset.as_ref(),
            );
    }

    pub async fn initialize_balance(
        &mut self,
        ctx: &NodeContext,
        exchange: impl AsRef<str>,
        symbol: impl AsRef<str>,
        initial_base: &Decimal,
        initial_quote: &Decimal,
        initial_price: &Decimal,
    ) -> Result<()> {
        self.get_or_insert(exchange.as_ref(), symbol.as_ref())
            .initialize_balance(ctx, initial_base, initial_quote, initial_price)
            .await?;
        Ok(())
    }

    pub async fn update_with_order(
        &mut self,
        ctx: &NodeContext,
        exchange: impl AsRef<str>,
        symbol: impl AsRef<str>,
        order: &Order,
    ) -> Result<()> {
        self.get_or_insert(exchange.as_ref(), symbol.as_ref())
            .update_with_order(ctx, order)
            .await?;

        Ok(())
    }
}

/// 节点统计数据
#[derive(Serialize, Deserialize, Debug, Default)]
#[allow(unused)]
pub struct SpotStatsData {
    pub exchange: String,                      // 交易所
    pub symbol: String,                        // 币种
    pub base_asset: String,                    // 基础币种
    pub quote_asset: String,                   // 计价币种
    pub initial_base_balance: Decimal,         // 初始化base资产余额
    pub initial_quote_balance: Decimal,        // 初始化quote资产余额
    pub initial_price: Decimal,                // 初始化价格
    pub maker_commission_rate: Decimal,        // maker手续费率
    pub taker_commission_rate: Decimal,        // taker手续费率
    pub base_asset_balance: Decimal,           // base资产持仓量
    pub quote_asset_balance: Decimal,          // quote资产持仓量
    pub avg_price: Decimal,                    // base资产持仓均价
    pub total_trades: u64,                     // 总交易次数
    pub buy_trades: u64,                       // 买入次数
    pub sell_trades: u64,                      // 卖出次数
    pub total_base_volume: Decimal,            // base资产交易量
    pub total_quote_volume: Decimal,           // quote资产交易量
    pub total_base_commission: Decimal,        // 总手续费
    pub total_quote_commission: Decimal,       // 总手续费
    pub realized_pnl: Decimal,                 // 已实现盈亏
    pub win_trades: u64,                       // 盈利交易次数
    pub first_trade_at: Option<DateTime<Utc>>, // 第一笔交易时间
    pub last_trade_at: Option<DateTime<Utc>>,  // 最后一笔交易时间
}

#[allow(unused)]
impl SpotStatsData {
    fn new() -> Self {
        SpotStatsData::default()
    }

    pub fn setup(&mut self, exchange: &str, symbol: &str, base_asset: &str, quote_asset: &str) {
        self.exchange = exchange.into();
        self.symbol = symbol.into();
        self.base_asset = base_asset.into();
        self.quote_asset = quote_asset.into();
    }

    fn params<'a>(&'a self, workflow_id: &'a str, node_id: i16) -> SpotStatsQuery<'a> {
        SpotStatsQuery::builder()
            .workflow_id(workflow_id)
            .node_id(node_id)
            .exchange(&self.exchange)
            .symbol(&self.symbol)
            .build()
    }

    async fn initialize_balance(
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
            &ctx.db,
            &ctx.node_name,
            &self.base_asset,
            &self.quote_asset,
            &self.params(&ctx.workflow_id, ctx.node_id),
        )
        .await?;

        Ok(())
    }

    async fn update_with_order(&mut self, ctx: &NodeContext, order: &Order) -> Result<()> {
        let now = Utc::now();
        let base_asset_amount = order.base_asset_amount()?;
        let quote_asset_amount = order.quote_asset_amount()?;
        let base_commission = order.base_commission(&self.maker_commission_rate)?;
        let quote_commission = order.quote_commission(&self.maker_commission_rate)?;
        let order_avg_price = order.avg_price.parse::<Decimal>()?;

        self.total_trades += 1;
        self.total_base_volume += base_asset_amount;
        self.total_quote_volume += quote_asset_amount;

        // 更新第一笔交易时间和最后一笔交易时间
        if self.first_trade_at.is_none() {
            self.first_trade_at = Some(now);
        }
        self.last_trade_at = Some(now);

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

        let params = self.params(&ctx.workflow_id, ctx.node_id);

        self.save_strategy_spot_stats(
            &ctx.db,
            &ctx.node_name,
            &self.base_asset,
            &self.quote_asset,
            &params,
        )
        .await?;
        self.save_strategy_spot_position(
            &ctx.db,
            &ctx.node_name,
            &self.base_asset,
            &self.quote_asset,
            &params,
        )
        .await?;

        Ok(())
    }

    // 计算特定时间点的净值
    pub fn calculate_net_value(
        &self,
        positions: &[StrategySpotPosition],
        klines: &[Kline],
    ) -> Result<Vec<NetValue>> {
        // 初始资产价值
        let initial_value = (self.initial_base_balance * self.initial_price
            + self.initial_quote_balance)
            .to_string()
            .parse::<f64>()?;

        let pos_len = positions.len();
        let kline_len = klines.len();

        let mut pos_timestamps = Vec::with_capacity(pos_len);
        let mut pos_base_balances = Vec::with_capacity(pos_len);
        let mut pos_quote_balances = Vec::with_capacity(pos_len);
        let mut kline_timestamps = Vec::with_capacity(kline_len);
        let mut kline_close_prices = Vec::with_capacity(kline_len);

        for p in positions {
            pos_timestamps.push(p.created_at.timestamp());
            pos_base_balances.push(p.base_asset_balance.to_f64().unwrap_or_default());
            pos_quote_balances.push(p.quote_asset_balance.to_f64().unwrap_or_default());
        }

        for k in klines {
            kline_timestamps.push(k.open_time.timestamp());
            kline_close_prices.push(k.close_price.to_f64().unwrap_or_default());
        }

        let pos_df = df!(
            "timestamp" => pos_timestamps,
            "base_balance" => pos_base_balances,
            "quote_balance" => pos_quote_balances,
        )?;

        let kline_df = df!(
            "timestamp" => kline_timestamps,
            "close" => kline_close_prices,
        )?;

        let df = kline_df
            // 合并数据
            .join(
                &pos_df,
                ["timestamp"],
                ["timestamp"],
                JoinArgs::new(JoinType::Left),
            )?
            // 排序
            .sort(["timestamp"], SortMultipleOptions::default())?
            .lazy()
            //  向前填充缺失的数据
            .with_columns([
                col("base_balance").fill_null_with_strategy(FillNullStrategy::Forward(None)),
                col("quote_balance").fill_null_with_strategy(FillNullStrategy::Forward(None)),
            ])
            // 计算资产价值
            .with_column(
                (col("base_balance") * col("close") + col("quote_balance")).alias("total_value"),
            )
            // 计算净值
            .with_column((col("total_value") / lit(initial_value)).alias("net_value"))
            // 计算最大净值
            .with_column(col("net_value").cum_max(false).alias("max_net_value"))
            // 计算回撤
            .with_column((lit(1.0) - col("net_value") / col("max_net_value")).alias("drawdown"))
            .collect()?;

        Ok(itertools::izip!(
            df.column("timestamp")?.i64()?.into_iter().flatten(),
            df.column("total_value")?.f64()?.into_iter().flatten(),
            df.column("net_value")?.f64()?.into_iter().flatten(),
            df.column("drawdown")?.f64()?.into_iter().flatten()
        )
        .map(|(timestamp, value, net_value, drawdown)| {
            NetValue::new(timestamp, value, net_value, drawdown)
        })
        .collect())
    }

    // 获取最大回撤
    pub fn get_max_drawdown(net_values: &[NetValue]) -> Decimal {
        net_values
            .iter()
            .map(|r| r.drawdown)
            .max()
            .unwrap_or(Decimal::ZERO)
    }

    // 已实现盈亏
    pub fn realized_pnl(&self) -> Decimal {
        self.realized_pnl
    }

    // 未实现盈亏
    pub fn unrealized_pnl(&self, price: &Decimal) -> Decimal {
        let cost = self.base_asset_balance * self.avg_price;
        let maybe_sell =
            self.base_asset_balance * price * (Decimal::ONE - self.maker_commission_rate);
        maybe_sell - cost
    }

    // 总收益率
    pub fn total_return(&self, current_price: &Decimal) -> Decimal {
        (self.realized_pnl() + self.unrealized_pnl(current_price))
            / (self.initial_quote_balance + self.initial_base_balance * current_price)
    }

    // 年化收益率 = (1 + 总收益率)^(365/交易天数) - 1
    pub fn annualized_return(&self, current_price: &Decimal) -> Decimal {
        let (first_time, last_time) = match (self.first_trade_at, self.last_trade_at) {
            (Some(first_time), Some(last_time)) => (first_time, last_time),
            (Some(first_time), None) => (first_time, Utc::now()),
            _ => return Decimal::ZERO,
        };

        // 计算交易天数
        let trading_days = (last_time - first_time).num_days();

        if trading_days == 0 {
            return Decimal::ZERO;
        }

        // 计算总收益率
        let total_return = self.total_return(current_price);

        // 年化收益率
        let days_ratio = Decimal::from(365) / Decimal::from(trading_days);
        (Decimal::ONE + total_return).powf(days_ratio.to_f64().unwrap_or(0.0)) - Decimal::ONE
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
            .exchange(params.exchange)
            .symbol(params.symbol)
            .base_asset(base_asset)
            .quote_asset(quote_asset)
            .base_asset_balance(self.base_asset_balance)
            .quote_asset_balance(self.quote_asset_balance)
            .realized_pnl(self.realized_pnl)
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
            .exchange(params.exchange)
            .symbol(params.symbol)
            .base_asset(base_asset)
            .quote_asset(quote_asset)
            .initial_base_balance(self.initial_base_balance)
            .initial_quote_balance(self.initial_quote_balance)
            .initial_price(self.initial_price)
            .maker_commission_rate(self.maker_commission_rate)
            .taker_commission_rate(self.taker_commission_rate)
            .base_asset_balance(self.base_asset_balance)
            .quote_asset_balance(self.quote_asset_balance)
            .avg_price(self.avg_price)
            .total_trades(self.total_trades as i64)
            .buy_trades(self.buy_trades as i64)
            .sell_trades(self.sell_trades as i64)
            .total_base_volume(self.total_base_volume)
            .total_quote_volume(self.total_quote_volume)
            .total_base_commission(self.total_base_commission)
            .total_quote_commission(self.total_quote_commission)
            .realized_pnl(self.realized_pnl)
            .win_trades(self.win_trades as i64)
            .build();

        strategy_spot_stats::create_or_update(db, &data).await?;

        Ok(())
    }
}

#[derive(Debug)]
pub struct NetValue {
    pub timestamp: i64,     // 时间戳
    pub value: Decimal,     // 总资产价值
    pub net_value: Decimal, // 净值
    pub drawdown: Decimal,  // 回撤
}

impl NetValue {
    pub fn new(timestamp: i64, value: f64, net_value: f64, drawdown: f64) -> Self {
        NetValue {
            timestamp,
            value: value.try_into().unwrap_or_default(),
            net_value: net_value.try_into().unwrap_or_default(),
            drawdown: drawdown.try_into().unwrap_or_default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{DateTime, Utc};
    use comfy_quant_base::secs_to_datetime;
    use comfy_quant_exchange::client::spot_client::base::{
        Order, OrderSide, OrderStatus, OrderType,
    };
    use rust_decimal_macros::dec;
    use std::{str::FromStr, sync::Arc};

    #[test]
    fn test_net_value_calculation() -> Result<()> {
        let workflow_id = "test_workflow";
        let node_id = 1;
        let node_name = "test_node";
        let exchange = "binance";
        let market = "spot";
        let symbol = "BTC/USDT";
        let base_asset = "BTC";
        let quote_asset = "USDT";

        // 初始资产: 1 BTC + 10000 USDT
        let initial_base = Decimal::ONE;
        let initial_quote = dec!(10000);
        let initial_price = dec!(50000);

        // 创建仓位快照序列
        let positions = vec![
            StrategySpotPosition::builder()
                .workflow_id(workflow_id)
                .node_id(node_id)
                .node_name(node_name)
                .exchange(exchange)
                .symbol(symbol)
                .base_asset(base_asset)
                .quote_asset(quote_asset)
                .base_asset_balance(Decimal::ONE)
                .quote_asset_balance(dec!(10000))
                .realized_pnl(dec!(0))
                .created_at(DateTime::<Utc>::from_timestamp(1000, 0).unwrap())
                .build(),
            StrategySpotPosition::builder()
                .workflow_id(workflow_id)
                .node_id(node_id)
                .node_name(node_name)
                .exchange(exchange)
                .symbol(symbol)
                .base_asset(base_asset)
                .quote_asset(quote_asset)
                .base_asset_balance(dec!(0.5))
                .quote_asset_balance(dec!(35000))
                .realized_pnl(dec!(0))
                .created_at(DateTime::<Utc>::from_timestamp(2000, 0).unwrap())
                .build(),
        ];

        let klines = vec![
            Kline::builder()
                .exchange(exchange)
                .market(market)
                .symbol(symbol)
                .interval("1m")
                .open_time(secs_to_datetime(1000)?)
                .open_price(dec!(50000))
                .high_price(dec!(50000))
                .low_price(dec!(50000))
                .close_price(dec!(50000))
                .volume(dec!(0))
                .build(),
            Kline::builder()
                .exchange(exchange)
                .market(market)
                .symbol(symbol)
                .interval("1m")
                .open_time(secs_to_datetime(1500)?)
                .open_price(dec!(50000))
                .high_price(dec!(50000))
                .low_price(dec!(45000))
                .close_price(dec!(45000))
                .volume(dec!(0))
                .build(),
            Kline::builder()
                .exchange(exchange)
                .market(market)
                .symbol(symbol)
                .interval("1m")
                .open_time(secs_to_datetime(2000)?)
                .open_price(dec!(48000))
                .high_price(dec!(48000))
                .low_price(dec!(48000))
                .close_price(dec!(48000))
                .volume(dec!(0))
                .build(),
            Kline::builder()
                .exchange(exchange)
                .market(market)
                .symbol(symbol)
                .interval("1m")
                .open_time(secs_to_datetime(2500)?)
                .open_price(dec!(48000))
                .high_price(dec!(48000))
                .low_price(dec!(48000))
                .close_price(dec!(52000))
                .volume(dec!(0))
                .build(),
            Kline::builder()
                .exchange(exchange)
                .market(market)
                .symbol(symbol)
                .interval("1m")
                .open_time(secs_to_datetime(3000)?)
                .open_price(dec!(48000))
                .high_price(dec!(48000))
                .low_price(dec!(48000))
                .close_price(dec!(45000))
                .volume(dec!(0))
                .build(),
        ];

        let mut stats = SpotStatsData::new();
        stats.setup(exchange, symbol, base_asset, quote_asset);
        stats.initial_base_balance = initial_base;
        stats.initial_quote_balance = initial_quote;
        stats.base_asset_balance = initial_base;
        stats.quote_asset_balance = initial_quote;
        stats.initial_price = initial_price;

        // 计算净值
        let results = stats.calculate_net_value(&positions, &klines).unwrap();

        // 验证计算结果
        assert_eq!(results.len(), 5);

        // t=1000: 1 BTC * 50000 + 10000 = 60000, 净值 = 1.0
        assert_eq!(results[0].value, dec!(60000));
        assert_eq!(results[0].net_value, Decimal::ONE);
        assert_eq!(results[0].drawdown, dec!(0));

        // t=1500: 1 BTC * 45000 + 10000 = 55000, 净值 = 0.9167
        assert_eq!(results[1].value, dec!(55000));
        assert_eq!(
            (results[1].net_value * dec!(10000)).round() / dec!(10000),
            dec!(0.9167)
        );
        assert_eq!(
            (results[1].drawdown * dec!(10000)).round() / dec!(10000),
            dec!(0.0833)
        );

        // t=2000: 0.5 BTC * 48000 + 35000 = 59000, 净值 = 0.9833
        assert_eq!(results[2].value, dec!(59000));
        assert_eq!(
            (results[2].net_value * dec!(10000)).round() / dec!(10000),
            dec!(0.9833)
        );
        assert_eq!(
            (results[2].drawdown * dec!(10000)).round() / dec!(10000),
            dec!(0.0167)
        );

        // t=2500: 0.5 BTC * 52000 + 35000 = 61000, 净值 = 1.0167
        assert_eq!(results[3].value, dec!(61000));
        assert_eq!(
            (results[3].net_value * dec!(10000)).round() / dec!(10000),
            dec!(1.0167)
        );
        assert_eq!(
            (results[3].drawdown * dec!(10000)).round() / dec!(10000),
            dec!(0)
        );

        // t=3000: 0.5 BTC * 45000 + 35000 = 57500, 净值 = 0.9583
        assert_eq!(results[4].value, dec!(57500));
        assert_eq!(
            (results[4].net_value * dec!(10000)).round() / dec!(10000),
            dec!(0.9583)
        );
        assert_eq!(
            (results[4].drawdown * dec!(10000)).round() / dec!(10000),
            dec!(0.0574)
        );

        Ok(())
    }

    fn create_test_order(side: OrderSide, price: &str, quantity: &str) -> Order {
        Order::builder()
            .order_id("test_order")
            .client_order_id("test_client_order")
            .symbol("BTC/USDT")
            .order_side(side)
            .order_status(OrderStatus::Filled)
            .price(price)
            .orig_qty(quantity)
            .executed_qty(quantity)
            .cumulative_quote_qty(
                (Decimal::from_str(price).unwrap() * Decimal::from_str(quantity).unwrap())
                    .to_string(),
            )
            .avg_price(price)
            .exchange("test")
            .base_asset("BTC")
            .quote_asset("USDT")
            .order_type(OrderType::Limit)
            .time(0)
            .update_time(0)
            .build()
    }

    #[test]
    fn test_spot_stats_data_setup() {
        let mut data = SpotStatsData::new();
        data.setup("binance", "BTC/USDT", "BTC", "USDT");

        assert_eq!(data.exchange, "binance");
        assert_eq!(data.symbol, "BTC/USDT");
        assert_eq!(data.base_asset, "BTC");
        assert_eq!(data.quote_asset, "USDT");
    }

    #[sqlx::test(migrator = "comfy_quant_database::MIGRATOR")]
    async fn test_spot_stats_data_update_with_buy_order(db: PgPool) {
        let mut data = SpotStatsData::new();
        data.setup("binance", "BTC/USDT", "BTC", "USDT");
        data.maker_commission_rate = dec!(0.001);
        data.quote_asset_balance = dec!(10000);

        let order = create_test_order(OrderSide::Buy, "50000", "0.1");

        // 模拟数据库连接和上下文
        let db = Arc::new(db);
        let workflow_id = "test_workflow";
        let node_id = 1_i16;
        let node_name = "test_node";
        let ctx = NodeContext::new(db, workflow_id, node_id, node_name);

        // 更新订单信息
        let result = data.update_with_order(ctx, &order).await;
        assert!(result.is_ok());

        // 验证数据更新
        assert_eq!(data.total_trades, 1);
        assert_eq!(data.buy_trades, 1);
        assert_eq!(data.sell_trades, 0);
        assert_eq!(data.base_asset_balance, dec!(0.0999)); // 0.1 - 0.1 * 0.001
        assert_eq!(data.quote_asset_balance, dec!(5000)); // 10000 - 50000 * 0.1
        assert_eq!(data.avg_price, dec!(50000));
    }

    #[sqlx::test(migrator = "comfy_quant_database::MIGRATOR")]
    async fn test_spot_stats_data_update_with_sell_order(db: PgPool) {
        let mut data = SpotStatsData::new();
        data.setup("binance", "BTC/USDT", "BTC", "USDT");
        data.maker_commission_rate = dec!(0.001);
        data.base_asset_balance = dec!(1.0);
        data.avg_price = dec!(45000);

        let order = create_test_order(OrderSide::Sell, "50000", "0.1");

        // 模拟数据库连接和上下文
        let db = Arc::new(db);
        let workflow_id = "test_workflow";
        let node_id = 1_i16;
        let node_name = "test_node";
        let ctx = NodeContext::new(db, workflow_id, node_id, node_name);

        // 更新订单信息
        let result = data.update_with_order(ctx, &order).await;
        assert!(result.is_ok());

        // 验证数据更新
        assert_eq!(data.total_trades, 1);
        assert_eq!(data.buy_trades, 0);
        assert_eq!(data.sell_trades, 1);
        assert_eq!(data.base_asset_balance, dec!(0.9));
        assert_eq!(data.quote_asset_balance, dec!(4995)); // 5000 * 0.999

        // 验证盈亏计算
        let expected_pnl = dec!(4995) - dec!(0.1) * dec!(45000);
        assert_eq!(data.realized_pnl, expected_pnl);
        assert_eq!(data.win_trades, 1);
    }

    #[test]
    fn test_spot_stats_data_pnl_calculations() {
        let mut data = SpotStatsData::new();
        data.setup("binance", "BTC/USDT", "BTC", "USDT");
        data.maker_commission_rate = dec!(0.001);
        data.base_asset_balance = dec!(1.0);
        data.avg_price = dec!(45000);

        // 测试未实现盈亏计算
        let current_price = dec!(50000);
        let expected_unrealized_pnl =
            dec!(1.0) * current_price * (Decimal::ONE - dec!(0.001)) - dec!(1.0) * dec!(45000);
        assert_eq!(data.unrealized_pnl(&current_price), expected_unrealized_pnl);

        // 测试已实现盈亏
        assert_eq!(data.realized_pnl(), dec!(0));
    }

    #[test]
    fn test_spot_stats_get_or_insert() {
        let exchange = "Binance";
        let symbol = "BTC/USDT";

        let mut stats = SpotStats::new();

        let data = stats.get_or_insert(exchange, symbol);
        assert_eq!(data.total_trades, 0);
        assert_eq!(data.buy_trades, 0);
        assert_eq!(data.sell_trades, 0);

        // 测试重复获取相同的key
        let data2 = stats.get_or_insert(exchange, symbol);
        assert_eq!(data2.total_trades, 0);
    }
}
