use anyhow::Result;
use bon::Builder;
use chrono::{DateTime, Utc};
use comfy_quant_base::{Exchange, Symbol};
use rust_decimal::Decimal;
use sqlx::{postgres::PgPool, FromRow};

#[derive(Debug, FromRow)]
pub struct StrategySpotPosition {
    pub id: i32,                      // 主键ID
    pub workflow_id: String,          // 工作流ID
    pub node_id: i16,                 // 策略节点ID
    pub node_name: String,            // 策略节点名称
    pub exchange: Exchange,           // 交易所
    pub symbol: Symbol,               // 交易对
    pub base_asset: String,           // 基础资产
    pub quote_asset: String,          // 计价资产
    pub base_asset_balance: Decimal,  // 基础资产持仓量
    pub quote_asset_balance: Decimal, // 计价资产持仓量
    pub realized_pnl: Decimal,        // 已实现盈亏
    pub created_at: DateTime<Utc>,    // 创建时间
}

#[derive(Builder)]
#[builder(on(_, into))]
pub struct CreateSpotPositionParams {
    pub workflow_id: String,          // 工作流ID
    pub node_id: i16,                 // 策略节点ID
    pub node_name: String,            // 策略节点名称
    pub exchange: Exchange,           // 交易所
    pub symbol: Symbol,               // 交易对
    pub base_asset: String,           // 基础资产
    pub quote_asset: String,          // 计价资产
    pub base_asset_balance: Decimal,  // 基础资产持仓量
    pub quote_asset_balance: Decimal, // 计价资产持仓量
    pub realized_pnl: Decimal,        // 已实现盈亏
}

pub async fn create(db: &PgPool, data: CreateSpotPositionParams) -> Result<StrategySpotPosition> {
    let strategy_spot_position = sqlx::query_as!(
        StrategySpotPosition,
        r#"
        INSERT INTO strategy_spot_positions (
            workflow_id, node_id, node_name, exchange, symbol, base_asset, quote_asset, base_asset_balance, quote_asset_balance, realized_pnl, created_at
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, NOW())
        RETURNING *
        "#,
        data.workflow_id,
        data.node_id,
        data.node_name,
        data.exchange.as_ref(),
        data.symbol.as_ref(),
        data.base_asset,
        data.quote_asset,
        data.base_asset_balance,
        data.quote_asset_balance,
        data.realized_pnl,
    )
    .fetch_one(db)
    .await?;

    Ok(strategy_spot_position)
}

pub async fn list(
    db: &PgPool,
    workflow_id: &str,
    node_id: i16,
    exchange: Exchange,
    symbol: &Symbol,
    start_datetime: &DateTime<Utc>,
    end_datetime: &DateTime<Utc>,
) -> Result<Vec<StrategySpotPosition>> {
    let result = sqlx::query_as!(
        StrategySpotPosition,
        r#"
        SELECT * FROM strategy_spot_positions
            WHERE
                workflow_id = $1 AND
                node_id = $2 AND
                exchange = $3 AND
                symbol = $4 AND
                created_at BETWEEN $5 AND $6
            ORDER BY created_at ASC
        "#,
        workflow_id,
        node_id,
        exchange.as_ref(),
        symbol.as_ref(),
        start_datetime,
        end_datetime,
    )
    .fetch_all(db)
    .await?;

    Ok(result)
}

#[cfg(test)]
mod tests {
    use comfy_quant_base::secs_to_datetime;
    use rust_decimal_macros::dec;

    use super::*;

    #[sqlx::test(migrator = "crate::MIGRATOR")]
    async fn test_strategy_spot_position_create(db: PgPool) -> Result<()> {
        let data = CreateSpotPositionParams::builder()
            .workflow_id("jEnbRDqQu4UN6y7cgQgp6")
            .node_id(1_i16)
            .node_name("SpotGrid")
            .exchange(Exchange::Binance)
            .symbol("BTCUSDT")
            .base_asset("BTC")
            .quote_asset("USDT")
            .base_asset_balance(Decimal::from(1))
            .quote_asset_balance(Decimal::from(1000))
            .realized_pnl(Decimal::from(0))
            .build();

        let strategy_spot_position = create(&db, data).await?;

        assert_eq!(strategy_spot_position.id, 1);
        assert_eq!(strategy_spot_position.workflow_id, "jEnbRDqQu4UN6y7cgQgp6");
        assert_eq!(strategy_spot_position.node_id, 1);
        assert_eq!(strategy_spot_position.node_name, "SpotGrid");
        assert_eq!(strategy_spot_position.exchange, Exchange::Binance);
        assert_eq!(strategy_spot_position.symbol, "BTCUSDT".into());
        assert_eq!(strategy_spot_position.base_asset, "BTC");
        assert_eq!(strategy_spot_position.quote_asset, "USDT");
        assert_eq!(strategy_spot_position.base_asset_balance, "1".parse()?);
        assert_eq!(strategy_spot_position.quote_asset_balance, "1000".parse()?);

        Ok(())
    }

    #[sqlx::test(migrator = "crate::MIGRATOR")]
    async fn test_strategy_spot_position_list(db: PgPool) -> Result<()> {
        let data = CreateSpotPositionParams::builder()
            .workflow_id("jEnbRDqQu4UN6y7cgQgp6")
            .node_id(1_i16)
            .node_name("SpotGrid")
            .exchange(Exchange::Binance)
            .symbol("BTCUSDT")
            .base_asset("BTC")
            .quote_asset("USDT")
            .base_asset_balance(dec!(1))
            .quote_asset_balance(dec!(1000))
            .realized_pnl(dec!(0))
            .build();

        create(&db, data).await?;

        let start_datetime = secs_to_datetime(0)?;
        let end_datetime = secs_to_datetime(10000000000_i64)?;
        let result = list(
            &db,
            "jEnbRDqQu4UN6y7cgQgp6",
            1,
            Exchange::Binance,
            &"BTCUSDT".into(),
            &start_datetime,
            &end_datetime,
        )
        .await?;

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].id, 1);
        assert_eq!(result[0].workflow_id, "jEnbRDqQu4UN6y7cgQgp6");
        assert_eq!(result[0].node_id, 1);
        assert_eq!(result[0].node_name, "SpotGrid");
        assert_eq!(result[0].exchange, Exchange::Binance);
        assert_eq!(result[0].symbol, "BTCUSDT".into());
        assert_eq!(result[0].base_asset, "BTC");
        assert_eq!(result[0].quote_asset, "USDT");
        assert_eq!(result[0].base_asset_balance, dec!(1));
        assert_eq!(result[0].quote_asset_balance, dec!(1000));
        assert_eq!(result[0].realized_pnl, dec!(0));

        Ok(())
    }
}
