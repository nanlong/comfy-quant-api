use crate::SpotStatsQuery;
use anyhow::Result;
use bon::Builder;
use chrono::{DateTime, Utc};
use comfy_quant_base::{Exchange, Symbol};
use rust_decimal::Decimal;
use sqlx::{postgres::PgPool, FromRow};

#[derive(Debug, FromRow)]
pub struct StrategySpotStats {
    pub id: i32,                         // 主键ID
    pub workflow_id: String,             // 工作流ID
    pub node_id: i16,                    // 策略节点ID
    pub node_name: String,               // 策略节点名称
    pub exchange: Exchange,              // 交易所
    pub symbol: Symbol,                  // 交易对
    pub base_asset: String,              // 基础资产
    pub quote_asset: String,             // 计价资产
    pub initial_base_balance: Decimal,   // 初始化基础资产余额
    pub initial_quote_balance: Decimal,  // 初始化计价资产余额
    pub initial_price: Decimal,          // 初始化价格
    pub maker_commission_rate: Decimal,  // maker手续费率
    pub taker_commission_rate: Decimal,  // taker手续费率
    pub base_asset_balance: Decimal,     // 基础资产持仓量
    pub quote_asset_balance: Decimal,    // 计价资产持仓量
    pub avg_price: Decimal,              // 基础资产持仓均价
    pub total_trades: i64,               // 总交易次数
    pub buy_trades: i64,                 // 买入次数
    pub sell_trades: i64,                // 卖出次数
    pub total_base_volume: Decimal,      // 基础资产交易量
    pub total_quote_volume: Decimal,     // 计价资产交易量
    pub total_base_commission: Decimal,  // 基础资产总手续费
    pub total_quote_commission: Decimal, // 计价资产总手续费
    pub realized_pnl: Decimal,           // 已实现盈亏
    pub win_trades: i64,                 // 盈利交易次数
    pub created_at: DateTime<Utc>,       // 创建时间
    pub updated_at: DateTime<Utc>,       // 更新时间
}

#[derive(Builder)]
#[builder(on(_, into))]
pub struct CreateSpotStatsParams {
    pub workflow_id: String,             // 工作流ID
    pub node_id: i16,                    // 策略节点ID
    pub node_name: String,               // 策略节点名称
    pub exchange: Exchange,              // 交易所
    pub symbol: Symbol,                  // 交易对
    pub base_asset: String,              // 基础资产
    pub quote_asset: String,             // 计价资产
    pub initial_base_balance: Decimal,   // 初始化基础资产余额
    pub initial_quote_balance: Decimal,  // 初始化计价资产余额
    pub initial_price: Decimal,          // 初始化价格
    pub maker_commission_rate: Decimal,  // maker手续费率
    pub taker_commission_rate: Decimal,  // taker手续费率
    pub base_asset_balance: Decimal,     // 基础资产持仓量
    pub quote_asset_balance: Decimal,    // 计价资产持仓量
    pub avg_price: Decimal,              // 基础资产持仓均价
    pub total_trades: i64,               // 总交易次数
    pub buy_trades: i64,                 // 买入次数
    pub sell_trades: i64,                // 卖出次数
    pub total_base_volume: Decimal,      // 基础资产交易量
    pub total_quote_volume: Decimal,     // 计价资产交易量
    pub total_base_commission: Decimal,  // 基础资产总手续费
    pub total_quote_commission: Decimal, // 计价资产总手续费
    pub realized_pnl: Decimal,           // 已实现盈亏
    pub win_trades: i64,                 // 盈利交易次数
}

#[derive(Builder)]
#[builder(on(_, into))]
pub struct UpdateSpotStatsParams {
    pub id: i32,                         // 主键ID
    pub base_asset_balance: Decimal,     // 基础资产持仓量
    pub quote_asset_balance: Decimal,    // 计价资产持仓量
    pub avg_price: Decimal,              // 基础资产持仓均价
    pub total_trades: i64,               // 总交易次数
    pub buy_trades: i64,                 // 买入次数
    pub sell_trades: i64,                // 卖出次数
    pub total_base_volume: Decimal,      // 基础资产交易量
    pub total_quote_volume: Decimal,     // 计价资产交易量
    pub total_base_commission: Decimal,  // 基础资产总手续费
    pub total_quote_commission: Decimal, // 计价资产总手续费
    pub realized_pnl: Decimal,           // 已实现盈亏
    pub win_trades: i64,                 // 盈利交易次数
}

pub async fn create(db: &PgPool, data: CreateSpotStatsParams) -> Result<StrategySpotStats> {
    let strategy_position = sqlx::query_as!(
        StrategySpotStats,
        r#"
        INSERT INTO strategy_spot_stats (
            workflow_id, node_id, node_name, exchange, symbol, base_asset, quote_asset, initial_base_balance, initial_quote_balance, initial_price, maker_commission_rate, taker_commission_rate, base_asset_balance, quote_asset_balance, avg_price, total_trades, buy_trades, sell_trades, total_base_volume, total_quote_volume, total_base_commission, total_quote_commission, realized_pnl, win_trades, created_at, updated_at
        )
        VALUES (
            $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20, $21, $22, $23, $24, NOW(), NOW()
        )
        RETURNING *
        "#,
        data.workflow_id,
        data.node_id,
        data.node_name,
        data.exchange.as_ref(),
        data.symbol.as_ref(),
        data.base_asset,
        data.quote_asset,
        data.initial_base_balance,
        data.initial_quote_balance,
        data.initial_price,
        data.maker_commission_rate,
        data.taker_commission_rate,
        data.base_asset_balance,
        data.quote_asset_balance,
        data.avg_price,
        data.total_trades,
        data.buy_trades,
        data.sell_trades,
        data.total_base_volume,
        data.total_quote_volume,
        data.total_base_commission,
        data.total_quote_commission,
        data.realized_pnl,
        data.win_trades,
    )
    .fetch_one(db)
    .await?;

    Ok(strategy_position)
}

pub async fn update(db: &PgPool, data: UpdateSpotStatsParams) -> Result<StrategySpotStats> {
    let strategy_position = sqlx::query_as!(
        StrategySpotStats,
        r#"
        UPDATE strategy_spot_stats
            SET
                base_asset_balance = $1,
                quote_asset_balance = $2,
                avg_price = $3,
                total_trades = $4,
                buy_trades = $5,
                sell_trades = $6,
                total_base_volume = $7,
                total_quote_volume = $8,
                total_base_commission = $9,
                total_quote_commission = $10,
                realized_pnl = $11,
                win_trades = $12,
                updated_at = NOW()
            WHERE id = $13
            RETURNING *
        "#,
        data.base_asset_balance,
        data.quote_asset_balance,
        data.avg_price,
        data.total_trades,
        data.buy_trades,
        data.sell_trades,
        data.total_base_volume,
        data.total_quote_volume,
        data.total_base_commission,
        data.total_quote_commission,
        data.realized_pnl,
        data.win_trades,
        data.id,
    )
    .fetch_one(db)
    .await?;

    Ok(strategy_position)
}

pub async fn get_by_unique_key(
    db: &PgPool,
    query: &SpotStatsQuery<'_>,
) -> Result<Option<StrategySpotStats>> {
    let strategy_spot_stats = sqlx::query_as!(
        StrategySpotStats,
        r#"
        SELECT * FROM strategy_spot_stats
            WHERE
                workflow_id = $1 AND
                node_id = $2 AND
                exchange = $3 AND
                symbol = $4
        "#,
        query.workflow_id,
        query.node_id,
        query.exchange.as_ref(),
        query.symbol.as_ref(),
    )
    .fetch_optional(db)
    .await?;

    Ok(strategy_spot_stats)
}

pub async fn create_or_update(
    db: &PgPool,
    data: CreateSpotStatsParams,
) -> Result<StrategySpotStats> {
    let strategy_spot_stats = sqlx::query_as!(
        StrategySpotStats,
        r#"
        INSERT INTO strategy_spot_stats (
            workflow_id, node_id, node_name, exchange, symbol, base_asset, quote_asset, initial_base_balance, initial_quote_balance, initial_price, maker_commission_rate, taker_commission_rate, base_asset_balance, quote_asset_balance, avg_price, total_trades, buy_trades, sell_trades, total_base_volume, total_quote_volume, total_base_commission, total_quote_commission, realized_pnl, win_trades, created_at, updated_at
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20, $21, $22, $23, $24, NOW(), NOW())
        ON CONFLICT (workflow_id, node_id, node_name, exchange, symbol, base_asset, quote_asset)
        DO UPDATE SET
            base_asset_balance = EXCLUDED.base_asset_balance,
            quote_asset_balance = EXCLUDED.quote_asset_balance,
            avg_price = EXCLUDED.avg_price,
            total_trades = EXCLUDED.total_trades,
            buy_trades = EXCLUDED.buy_trades,
            sell_trades = EXCLUDED.sell_trades,
            total_base_volume = EXCLUDED.total_base_volume,
            total_quote_volume = EXCLUDED.total_quote_volume,
            total_base_commission = EXCLUDED.total_base_commission,
            total_quote_commission = EXCLUDED.total_quote_commission,
            realized_pnl = EXCLUDED.realized_pnl,
            win_trades = EXCLUDED.win_trades,
            updated_at = NOW()
        RETURNING *
        "#,
        data.workflow_id,
        data.node_id,
        data.node_name,
        data.exchange.as_ref(),
        data.symbol.as_ref(),
        data.base_asset,
        data.quote_asset,
        data.initial_base_balance,
        data.initial_quote_balance,
        data.initial_price,
        data.maker_commission_rate,
        data.taker_commission_rate,
        data.base_asset_balance,
        data.quote_asset_balance,
        data.avg_price,
        data.total_trades,
        data.buy_trades,
        data.sell_trades,
        data.total_base_volume,
        data.total_quote_volume,
        data.total_base_commission,
        data.total_quote_commission,
        data.realized_pnl,
        data.win_trades,
    )
    .fetch_one(db)
    .await?;

    Ok(strategy_spot_stats)
}

#[cfg(test)]
mod tests {
    use rust_decimal_macros::dec;

    use super::*;

    fn gen_strategy_spot_stats() -> Result<CreateSpotStatsParams> {
        let strategy_spot_stats = CreateSpotStatsParams::builder()
            .workflow_id("jEnbRDqQu4UN6y7cgQgp6")
            .node_id(1_i16)
            .node_name("SpotGrid")
            .exchange(Exchange::Binance)
            .symbol("BTCUSDT")
            .base_asset("BTC")
            .quote_asset("USDT")
            .initial_base_balance(dec!(1))
            .initial_quote_balance(dec!(1000))
            .initial_price(dec!(10000))
            .maker_commission_rate(dec!(0.001))
            .taker_commission_rate(dec!(0.001))
            .base_asset_balance(dec!(1))
            .quote_asset_balance(dec!(1000))
            .avg_price(dec!(10000))
            .total_trades(100)
            .buy_trades(50)
            .sell_trades(50)
            .total_base_volume(dec!(100))
            .total_quote_volume(dec!(100000))
            .total_base_commission(dec!(10))
            .total_quote_commission(dec!(1000))
            .realized_pnl(dec!(100))
            .win_trades(50)
            .build();

        Ok(strategy_spot_stats)
    }

    #[sqlx::test(migrator = "crate::MIGRATOR")]
    async fn test_strategy_spot_stats_create(db: PgPool) -> Result<()> {
        let data = gen_strategy_spot_stats()?;
        let strategy_spot_stats = create(&db, data).await?;

        assert_eq!(strategy_spot_stats.id, 1);
        assert_eq!(strategy_spot_stats.workflow_id, "jEnbRDqQu4UN6y7cgQgp6");
        assert_eq!(strategy_spot_stats.node_id, 1);
        assert_eq!(strategy_spot_stats.node_name, "SpotGrid");
        assert_eq!(strategy_spot_stats.exchange, Exchange::Binance);
        assert_eq!(strategy_spot_stats.symbol, "BTCUSDT".into());
        assert_eq!(strategy_spot_stats.base_asset, "BTC");
        assert_eq!(strategy_spot_stats.quote_asset, "USDT");
        assert_eq!(strategy_spot_stats.initial_base_balance, dec!(1));
        assert_eq!(strategy_spot_stats.initial_quote_balance, dec!(1000));
        assert_eq!(strategy_spot_stats.initial_price, dec!(10000));
        assert_eq!(strategy_spot_stats.maker_commission_rate, dec!(0.001));
        assert_eq!(strategy_spot_stats.taker_commission_rate, dec!(0.001));
        assert_eq!(strategy_spot_stats.base_asset_balance, dec!(1));
        assert_eq!(strategy_spot_stats.quote_asset_balance, dec!(1000));
        assert_eq!(strategy_spot_stats.avg_price, dec!(10000));
        assert_eq!(strategy_spot_stats.total_trades, 100);
        assert_eq!(strategy_spot_stats.buy_trades, 50);
        assert_eq!(strategy_spot_stats.sell_trades, 50);
        assert_eq!(strategy_spot_stats.total_base_volume, dec!(100));
        assert_eq!(strategy_spot_stats.total_quote_volume, dec!(100000));
        assert_eq!(strategy_spot_stats.total_base_commission, dec!(10));
        assert_eq!(strategy_spot_stats.total_quote_commission, dec!(1000));
        assert_eq!(strategy_spot_stats.realized_pnl, dec!(100));
        assert_eq!(strategy_spot_stats.win_trades, 50);

        Ok(())
    }

    #[sqlx::test(migrator = "crate::MIGRATOR")]
    async fn test_strategy_spot_stats_update(db: PgPool) -> Result<()> {
        let data = gen_strategy_spot_stats()?;
        let mut strategy_spot_stats = create(&db, data).await?;

        strategy_spot_stats.base_asset_balance = "2".parse()?;
        strategy_spot_stats.quote_asset_balance = "2000".parse()?;
        strategy_spot_stats.avg_price = "20000".parse()?;
        strategy_spot_stats.total_trades = 200;
        strategy_spot_stats.buy_trades = 100;
        strategy_spot_stats.sell_trades = 100;

        let data = UpdateSpotStatsParams::builder()
            .id(strategy_spot_stats.id)
            .base_asset_balance(dec!(2))
            .quote_asset_balance(dec!(2000))
            .avg_price(dec!(20000))
            .total_trades(200)
            .buy_trades(100)
            .sell_trades(100)
            .total_base_volume(dec!(200))
            .total_quote_volume(dec!(200000))
            .total_base_commission(dec!(20))
            .total_quote_commission(dec!(2000))
            .realized_pnl(dec!(200))
            .win_trades(100)
            .build();

        let strategy_spot_stats = update(&db, data).await?;

        assert_eq!(strategy_spot_stats.base_asset_balance, "2".parse()?);
        assert_eq!(strategy_spot_stats.quote_asset_balance, "2000".parse()?);
        assert_eq!(strategy_spot_stats.avg_price, "20000".parse()?);
        assert_eq!(strategy_spot_stats.total_trades, 200);
        assert_eq!(strategy_spot_stats.buy_trades, 100);
        assert_eq!(strategy_spot_stats.sell_trades, 100);

        Ok(())
    }

    #[sqlx::test(migrator = "crate::MIGRATOR")]
    async fn test_strategy_spot_stats_create_or_update(db: PgPool) -> Result<()> {
        let data = gen_strategy_spot_stats()?;
        let strategy_spot_stats = create_or_update(&db, data).await?;

        assert_eq!(strategy_spot_stats.id, 1);
        assert_eq!(strategy_spot_stats.base_asset_balance, dec!(1));

        let mut data = gen_strategy_spot_stats()?;
        data.base_asset_balance = "2".parse()?;
        let strategy_spot_stats = create_or_update(&db, data).await?;

        assert_eq!(strategy_spot_stats.id, 1);
        assert_eq!(strategy_spot_stats.base_asset_balance, "2".parse()?);

        Ok(())
    }

    #[sqlx::test(migrator = "crate::MIGRATOR")]
    async fn test_strategy_spot_stats_get_by_unique_key(db: PgPool) -> Result<()> {
        let data = gen_strategy_spot_stats()?;
        let data = create(&db, data).await?;

        let query = SpotStatsQuery::builder()
            .workflow_id(&data.workflow_id)
            .node_id(data.node_id)
            .exchange(&data.exchange)
            .symbol(&data.symbol)
            .build();

        let strategy_spot_stats = get_by_unique_key(&db, &query).await?;

        assert_eq!(strategy_spot_stats.is_some(), true);
        assert_eq!(strategy_spot_stats.unwrap().id, data.id);

        Ok(())
    }
}
