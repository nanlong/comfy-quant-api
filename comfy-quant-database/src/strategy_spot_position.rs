use anyhow::Result;
use bon::bon;
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use sqlx::{postgres::PgPool, FromRow};

#[derive(Debug, Default, FromRow)]
pub struct StrategySpotPosition {
    pub id: i32,                      // 主键ID
    pub workflow_id: String,          // 工作流ID
    pub node_id: i16,                 // 策略节点ID
    pub node_name: String,            // 策略节点名称
    pub exchange: String,             // 交易所
    pub symbol: String,               // 交易对
    pub base_asset: String,           // 基础资产
    pub quote_asset: String,          // 计价资产
    pub base_asset_balance: Decimal,  // 基础资产持仓量
    pub quote_asset_balance: Decimal, // 计价资产持仓量
    pub realized_pnl: Decimal,        // 已实现盈亏
    pub created_at: DateTime<Utc>,    // 创建时间
}

#[bon]
impl StrategySpotPosition {
    #[builder(on(String, into))]
    pub fn new(
        workflow_id: String,
        node_id: i16,
        node_name: String,
        exchange: String,
        symbol: String,
        base_asset: String,
        quote_asset: String,
        base_asset_balance: Decimal,
        quote_asset_balance: Decimal,
        realized_pnl: Decimal,
        created_at: Option<DateTime<Utc>>, // 方便测试
    ) -> Self {
        StrategySpotPosition {
            workflow_id,
            node_id,
            node_name,
            exchange,
            symbol,
            base_asset,
            quote_asset,
            base_asset_balance,
            quote_asset_balance,
            realized_pnl,
            created_at: created_at.unwrap_or_else(Utc::now),
            ..Default::default()
        }
    }
}

pub async fn create(db: &PgPool, data: &StrategySpotPosition) -> Result<StrategySpotPosition> {
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
        data.exchange,
        data.symbol,
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
    exchange: &str,
    symbol: &str,
    start_timestamp: i64,
    end_timestamp: i64,
) -> Result<Vec<StrategySpotPosition>> {
    let start_datetime = DateTime::<Utc>::from_timestamp(start_timestamp, 0)
        .ok_or_else(|| anyhow::anyhow!("Invalid start timestamp"))?;
    let end_datetime = DateTime::<Utc>::from_timestamp(end_timestamp, 0)
        .ok_or_else(|| anyhow::anyhow!("Invalid end timestamp"))?;

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
        exchange,
        symbol,
        start_datetime,
        end_datetime,
    )
    .fetch_all(db)
    .await?;

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn gen_strategy_spot_position() -> Result<StrategySpotPosition> {
        let strategy_spot_position = StrategySpotPosition::builder()
            .workflow_id("jEnbRDqQu4UN6y7cgQgp6")
            .node_id(1)
            .node_name("SpotGrid")
            .exchange("Binance")
            .symbol("BTCUSDT")
            .base_asset("BTC")
            .quote_asset("USDT")
            .base_asset_balance("1".parse()?)
            .quote_asset_balance("1000".parse()?)
            .realized_pnl("0".parse()?)
            .created_at(DateTime::<Utc>::from_timestamp(0, 0).unwrap())
            .build();

        Ok(strategy_spot_position)
    }

    #[sqlx::test(migrator = "crate::MIGRATOR")]
    async fn test_strategy_spot_position_create(db: PgPool) -> Result<()> {
        let strategy_spot_position = gen_strategy_spot_position()?;
        let strategy_spot_position = create(&db, &strategy_spot_position).await?;

        assert_eq!(strategy_spot_position.id, 1);
        assert_eq!(strategy_spot_position.workflow_id, "jEnbRDqQu4UN6y7cgQgp6");
        assert_eq!(strategy_spot_position.node_id, 1);
        assert_eq!(strategy_spot_position.node_name, "SpotGrid");
        assert_eq!(strategy_spot_position.exchange, "Binance");
        assert_eq!(strategy_spot_position.symbol, "BTCUSDT");
        assert_eq!(strategy_spot_position.base_asset, "BTC");
        assert_eq!(strategy_spot_position.quote_asset, "USDT");
        assert_eq!(strategy_spot_position.base_asset_balance, "1".parse()?);
        assert_eq!(strategy_spot_position.quote_asset_balance, "1000".parse()?);

        Ok(())
    }

    #[sqlx::test(migrator = "crate::MIGRATOR")]
    async fn test_strategy_spot_position_list(db: PgPool) -> Result<()> {
        let strategy_spot_position = gen_strategy_spot_position()?;
        create(&db, &strategy_spot_position).await?;

        let start_timestamp = 0;
        let end_timestamp = 10000000000;

        let result = list(
            &db,
            "jEnbRDqQu4UN6y7cgQgp6",
            1,
            "Binance",
            "BTCUSDT",
            start_timestamp,
            end_timestamp,
        )
        .await?;

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].id, 1);
        assert_eq!(result[0].workflow_id, strategy_spot_position.workflow_id);
        assert_eq!(result[0].node_id, strategy_spot_position.node_id);
        assert_eq!(result[0].node_name, strategy_spot_position.node_name);
        assert_eq!(result[0].exchange, strategy_spot_position.exchange);
        assert_eq!(result[0].symbol, strategy_spot_position.symbol);
        assert_eq!(result[0].base_asset, strategy_spot_position.base_asset);
        assert_eq!(result[0].quote_asset, strategy_spot_position.quote_asset);
        assert_eq!(
            result[0].base_asset_balance,
            strategy_spot_position.base_asset_balance
        );
        assert_eq!(
            result[0].quote_asset_balance,
            strategy_spot_position.quote_asset_balance
        );
        assert_eq!(result[0].realized_pnl, strategy_spot_position.realized_pnl);

        Ok(())
    }
}
