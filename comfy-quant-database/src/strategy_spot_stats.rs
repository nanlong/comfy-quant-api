use anyhow::Result;
use bon::{bon, Builder};
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use sqlx::{postgres::PgPool, FromRow};

#[derive(Debug, Default, FromRow)]
pub struct StrategySpotStats {
    pub id: i32,                         // 主键ID
    pub workflow_id: String,             // 工作流ID
    pub node_id: i16,                    // 策略节点ID
    pub node_name: String,               // 策略节点名称
    pub exchange: String,                // 交易所
    pub symbol: String,                  // 交易对
    pub base_asset: String,              // 基础资产
    pub quote_asset: String,             // 计价资产
    pub initial_base_balance: Decimal,   // 初始化基础资产余额
    pub initial_quote_balance: Decimal,  // 初始化计价资产余额
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

#[bon]
impl StrategySpotStats {
    #[builder(on(String, into))]
    pub fn new(
        workflow_id: String,
        node_id: i16,
        node_name: String,
        exchange: String,
        symbol: String,
        base_asset: String,
        quote_asset: String,
        initial_base_balance: Decimal,
        initial_quote_balance: Decimal,
        maker_commission_rate: Decimal,
        taker_commission_rate: Decimal,
        base_asset_balance: Decimal,
        quote_asset_balance: Decimal,
        avg_price: Decimal,
        total_trades: i64,
        buy_trades: i64,
        sell_trades: i64,
        total_base_volume: Decimal,
        total_quote_volume: Decimal,
        total_base_commission: Decimal,
        total_quote_commission: Decimal,
        realized_pnl: Decimal,
        win_trades: i64,
    ) -> Self {
        StrategySpotStats {
            workflow_id,
            node_id,
            node_name,
            exchange,
            symbol,
            base_asset,
            quote_asset,
            initial_base_balance,
            initial_quote_balance,
            maker_commission_rate,
            taker_commission_rate,
            base_asset_balance,
            quote_asset_balance,
            avg_price,
            total_trades,
            buy_trades,
            sell_trades,
            total_base_volume,
            total_quote_volume,
            total_base_commission,
            total_quote_commission,
            realized_pnl,
            win_trades,
            updated_at: Utc::now(),
            ..Default::default()
        }
    }
}

pub async fn create(db: &PgPool, data: &StrategySpotStats) -> Result<StrategySpotStats> {
    let strategy_position = sqlx::query_as!(
        StrategySpotStats,
        r#"
        INSERT INTO strategy_spot_stats (workflow_id, node_id, node_name, exchange, symbol, base_asset, quote_asset, initial_base_balance, initial_quote_balance, maker_commission_rate, taker_commission_rate, base_asset_balance, quote_asset_balance, avg_price, total_trades, buy_trades, sell_trades, total_base_volume, total_quote_volume, total_base_commission, total_quote_commission, realized_pnl, win_trades, created_at, updated_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20, $21, $22, $23, NOW(), NOW())
        RETURNING *
        "#,
        data.workflow_id,
        data.node_id,
        data.node_name,
        data.exchange,
        data.symbol,
        data.base_asset,
        data.quote_asset,
        data.initial_base_balance,
        data.initial_quote_balance,
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

pub async fn update(db: &PgPool, data: &StrategySpotStats) -> Result<StrategySpotStats> {
    let strategy_position = sqlx::query_as!(
        StrategySpotStats,
        r#"
        UPDATE strategy_spot_stats SET base_asset_balance = $1, quote_asset_balance = $2, avg_price = $3, total_trades = $4, buy_trades = $5, sell_trades = $6, total_base_volume = $7, total_quote_volume = $8, total_base_commission = $9, total_quote_commission = $10, realized_pnl = $11, win_trades = $12, updated_at = NOW() WHERE id = $13
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

#[derive(Debug, Builder)]
pub struct SpotStatsUniqueKey<'a> {
    pub workflow_id: &'a str,
    pub node_id: i16,
    pub node_name: &'a str,
    pub exchange: &'a str,
    pub symbol: &'a str,
    pub base_asset: &'a str,
    pub quote_asset: &'a str,
}

pub async fn get_by_unique_key(
    db: &PgPool,
    query: &SpotStatsUniqueKey<'_>,
) -> Result<Option<StrategySpotStats>> {
    let strategy_spot_stats = sqlx::query_as!(
        StrategySpotStats,
        r#"SELECT * FROM strategy_spot_stats WHERE workflow_id = $1 AND node_id = $2 AND node_name = $3 AND exchange = $4 AND symbol = $5 AND base_asset = $6 AND quote_asset = $7"#,
        query.workflow_id, query.node_id, query.node_name, query.exchange, query.symbol, query.base_asset, query.quote_asset
    )
    .fetch_optional(db)
    .await?;

    Ok(strategy_spot_stats)
}

pub async fn create_or_update(db: &PgPool, data: &StrategySpotStats) -> Result<StrategySpotStats> {
    let query = SpotStatsUniqueKey::builder()
        .workflow_id(&data.workflow_id)
        .node_id(data.node_id)
        .node_name(&data.node_name)
        .exchange(&data.exchange)
        .symbol(&data.symbol)
        .base_asset(&data.base_asset)
        .quote_asset(&data.quote_asset)
        .build();

    match get_by_unique_key(db, &query).await? {
        Some(mut existing) => {
            // 如果数据更新时间大于现有数据更新时间，则更新数据，返回更新后的数据
            if data.updated_at > existing.updated_at {
                existing.base_asset_balance = data.base_asset_balance;
                existing.quote_asset_balance = data.quote_asset_balance;
                existing.avg_price = data.avg_price;
                existing.total_trades = data.total_trades;
                existing.buy_trades = data.buy_trades;
                existing.sell_trades = data.sell_trades;
                existing.total_base_volume = data.total_base_volume;
                existing.total_quote_volume = data.total_quote_volume;
                existing.total_base_commission = data.total_base_commission;
                existing.total_quote_commission = data.total_quote_commission;
                existing.realized_pnl = data.realized_pnl;
                existing.win_trades = data.win_trades;
                update(db, &existing).await
            }
            // 如果数据更新时间小于现有数据更新时间，则不更新数据，返回现有数据
            else {
                Ok(existing)
            }
        }
        None => create(db, data).await,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn gen_strategy_spot_stats() -> Result<StrategySpotStats> {
        let strategy_spot_stats = StrategySpotStats::builder()
            .workflow_id("jEnbRDqQu4UN6y7cgQgp6")
            .node_id(1)
            .node_name("SpotGrid")
            .exchange("Binance")
            .symbol("BTCUSDT")
            .base_asset("BTC")
            .quote_asset("USDT")
            .initial_base_balance("1".parse::<Decimal>()?)
            .initial_quote_balance("1000".parse::<Decimal>()?)
            .maker_commission_rate("0.001".parse::<Decimal>()?)
            .taker_commission_rate("0.001".parse::<Decimal>()?)
            .base_asset_balance("1".parse::<Decimal>()?)
            .quote_asset_balance("1000".parse::<Decimal>()?)
            .avg_price("10000".parse::<Decimal>()?)
            .total_trades(100)
            .buy_trades(50)
            .sell_trades(50)
            .total_base_volume("100".parse::<Decimal>()?)
            .total_quote_volume("100000".parse::<Decimal>()?)
            .total_base_commission("10".parse::<Decimal>()?)
            .total_quote_commission("1000".parse::<Decimal>()?)
            .realized_pnl("100".parse::<Decimal>()?)
            .win_trades(50)
            .build();

        Ok(strategy_spot_stats)
    }

    #[sqlx::test(migrator = "crate::MIGRATOR")]
    async fn test_strategy_spot_stats_create(db: PgPool) -> Result<()> {
        let strategy_spot_stats = gen_strategy_spot_stats()?;
        let strategy_spot_stats = create(&db, &strategy_spot_stats).await?;

        assert_eq!(strategy_spot_stats.id, 1);
        assert_eq!(strategy_spot_stats.workflow_id, "jEnbRDqQu4UN6y7cgQgp6");
        assert_eq!(strategy_spot_stats.node_id, 1);
        assert_eq!(strategy_spot_stats.node_name, "SpotGrid");
        assert_eq!(strategy_spot_stats.exchange, "Binance");
        assert_eq!(strategy_spot_stats.symbol, "BTCUSDT");
        assert_eq!(strategy_spot_stats.base_asset, "BTC");
        assert_eq!(strategy_spot_stats.quote_asset, "USDT");
        assert_eq!(
            strategy_spot_stats.initial_base_balance,
            "1".parse::<Decimal>()?
        );
        assert_eq!(
            strategy_spot_stats.initial_quote_balance,
            "1000".parse::<Decimal>()?
        );
        assert_eq!(
            strategy_spot_stats.maker_commission_rate,
            "0.001".parse::<Decimal>()?
        );
        assert_eq!(
            strategy_spot_stats.taker_commission_rate,
            "0.001".parse::<Decimal>()?
        );
        assert_eq!(
            strategy_spot_stats.base_asset_balance,
            "1".parse::<Decimal>()?
        );
        assert_eq!(
            strategy_spot_stats.quote_asset_balance,
            "1000".parse::<Decimal>()?
        );
        assert_eq!(strategy_spot_stats.avg_price, "10000".parse::<Decimal>()?);
        assert_eq!(strategy_spot_stats.total_trades, 100);
        assert_eq!(strategy_spot_stats.buy_trades, 50);
        assert_eq!(strategy_spot_stats.sell_trades, 50);
        assert_eq!(
            strategy_spot_stats.total_base_volume,
            "100".parse::<Decimal>()?
        );
        assert_eq!(
            strategy_spot_stats.total_quote_volume,
            "100000".parse::<Decimal>()?
        );
        assert_eq!(
            strategy_spot_stats.total_base_commission,
            "10".parse::<Decimal>()?
        );
        assert_eq!(
            strategy_spot_stats.total_quote_commission,
            "1000".parse::<Decimal>()?
        );
        assert_eq!(strategy_spot_stats.realized_pnl, "100".parse::<Decimal>()?);
        assert_eq!(strategy_spot_stats.win_trades, 50);

        Ok(())
    }

    #[sqlx::test(migrator = "crate::MIGRATOR")]
    async fn test_strategy_spot_stats_update(db: PgPool) -> Result<()> {
        let strategy_spot_stats = gen_strategy_spot_stats()?;
        let mut strategy_spot_stats = create(&db, &strategy_spot_stats).await?;

        strategy_spot_stats.base_asset_balance = "2".parse::<Decimal>()?;
        strategy_spot_stats.quote_asset_balance = "2000".parse::<Decimal>()?;
        strategy_spot_stats.avg_price = "20000".parse::<Decimal>()?;
        strategy_spot_stats.total_trades = 200;
        strategy_spot_stats.buy_trades = 100;
        strategy_spot_stats.sell_trades = 100;

        let strategy_spot_stats = update(&db, &strategy_spot_stats).await?;

        assert_eq!(
            strategy_spot_stats.base_asset_balance,
            "2".parse::<Decimal>()?
        );
        assert_eq!(
            strategy_spot_stats.quote_asset_balance,
            "2000".parse::<Decimal>()?
        );
        assert_eq!(strategy_spot_stats.avg_price, "20000".parse::<Decimal>()?);
        assert_eq!(strategy_spot_stats.total_trades, 200);
        assert_eq!(strategy_spot_stats.buy_trades, 100);
        assert_eq!(strategy_spot_stats.sell_trades, 100);

        Ok(())
    }

    #[sqlx::test(migrator = "crate::MIGRATOR")]
    async fn test_strategy_spot_stats_create_or_update(db: PgPool) -> Result<()> {
        let strategy_spot_stats = gen_strategy_spot_stats()?;
        let strategy_spot_stats = create_or_update(&db, &strategy_spot_stats).await?;

        assert_eq!(strategy_spot_stats.id, 1);
        assert_eq!(
            strategy_spot_stats.base_asset_balance,
            "1".parse::<Decimal>()?
        );

        let mut strategy_spot_stats = gen_strategy_spot_stats()?;
        strategy_spot_stats.base_asset_balance = "2".parse::<Decimal>()?;
        let strategy_spot_stats = create_or_update(&db, &strategy_spot_stats).await?;

        assert_eq!(strategy_spot_stats.id, 1);
        assert_eq!(
            strategy_spot_stats.base_asset_balance,
            "2".parse::<Decimal>()?
        );

        Ok(())
    }

    #[sqlx::test(migrator = "crate::MIGRATOR")]
    async fn test_strategy_spot_stats_get_by_unique_key(db: PgPool) -> Result<()> {
        let strategy_spot_stats = gen_strategy_spot_stats()?;
        let data = create(&db, &strategy_spot_stats).await?;

        let query = SpotStatsUniqueKey::builder()
            .workflow_id(&data.workflow_id)
            .node_id(data.node_id)
            .node_name(&data.node_name)
            .exchange(&data.exchange)
            .symbol(&data.symbol)
            .base_asset(&data.base_asset)
            .quote_asset(&data.quote_asset)
            .build();

        let strategy_spot_stats = get_by_unique_key(&db, &query).await?;

        assert_eq!(strategy_spot_stats.is_some(), true);
        assert_eq!(strategy_spot_stats.unwrap().id, data.id);

        Ok(())
    }
}
