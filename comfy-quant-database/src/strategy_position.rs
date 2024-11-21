use anyhow::Result;
use bon::bon;
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use sqlx::{postgres::PgPool, FromRow};

#[derive(Debug, Default, FromRow)]
pub struct StrategyPosition {
    pub id: i32,
    pub workflow_id: String,
    pub node_id: i16,
    pub exchange: String,
    pub market: String,
    pub symbol: String,
    pub base_asset: String,
    pub quote_asset: String,
    pub base_asset_balance: Decimal,
    pub quote_asset_balance: Decimal,
    pub created_at: DateTime<Utc>,
}

#[bon]
impl StrategyPosition {
    #[builder(on(String, into))]
    fn new(
        workflow_id: String,
        node_id: i16,
        exchange: String,
        market: String,
        symbol: String,
        base_asset: String,
        quote_asset: String,
        base_asset_balance: Decimal,
        quote_asset_balance: Decimal,
    ) -> Self {
        StrategyPosition {
            workflow_id,
            node_id,
            exchange,
            market,
            symbol,
            base_asset,
            quote_asset,
            base_asset_balance,
            quote_asset_balance,
            ..Default::default()
        }
    }
}

pub async fn insert(
    pool: &PgPool,
    strategy_position: &StrategyPosition,
) -> Result<StrategyPosition> {
    let strategy_position = sqlx::query_as!(
        StrategyPosition,
        r#"
        INSERT INTO strategy_positions (workflow_id, node_id, exchange, market, symbol, base_asset, quote_asset, base_asset_balance, quote_asset_balance, created_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, NOW())
        RETURNING *
        "#,
        strategy_position.workflow_id,
        strategy_position.node_id,
        strategy_position.exchange,
        strategy_position.market,
        strategy_position.symbol,
        strategy_position.base_asset,
        strategy_position.quote_asset,
        strategy_position.base_asset_balance,
        strategy_position.quote_asset_balance,
    )
    .fetch_one(pool)
    .await?;

    Ok(strategy_position)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[sqlx::test(migrator = "crate::MIGRATOR")]
    async fn test_strategy_position_insert_with_default(pool: PgPool) -> Result<()> {
        let strategy_position = insert(&pool, &StrategyPosition::default()).await?;

        assert_eq!(strategy_position.id, 1);
        assert_eq!(strategy_position.workflow_id, "");
        assert_eq!(strategy_position.node_id, 0);
        assert_eq!(strategy_position.exchange, "");
        assert_eq!(strategy_position.market, "");
        assert_eq!(strategy_position.symbol, "");
        assert_eq!(strategy_position.base_asset, "");
        assert_eq!(strategy_position.quote_asset, "");
        assert_eq!(strategy_position.base_asset_balance, Decimal::ZERO);
        assert_eq!(strategy_position.quote_asset_balance, Decimal::ZERO);

        Ok(())
    }

    #[sqlx::test(migrator = "crate::MIGRATOR")]
    async fn test_strategy_position_insert(pool: PgPool) -> Result<()> {
        let workflow_id = "jEnbRDqQu4UN6y7cgQgp6";
        let base_asset_balance = "1".parse::<Decimal>()?;
        let quote_asset_balance = "1000".parse::<Decimal>()?;
        let strategy_position = StrategyPosition::builder()
            .workflow_id(workflow_id)
            .node_id(1)
            .exchange("Binance")
            .market("spot")
            .symbol("BTCUSDT")
            .base_asset("BTC")
            .quote_asset("USDT")
            .base_asset_balance(base_asset_balance)
            .quote_asset_balance(quote_asset_balance)
            .build();

        let strategy_position = insert(&pool, &strategy_position).await?;

        assert_eq!(strategy_position.id, 1);
        assert_eq!(strategy_position.workflow_id, workflow_id);
        assert_eq!(strategy_position.node_id, 1);
        assert_eq!(strategy_position.exchange, "Binance");
        assert_eq!(strategy_position.market, "spot");
        assert_eq!(strategy_position.symbol, "BTCUSDT");
        assert_eq!(strategy_position.base_asset, "BTC");
        assert_eq!(strategy_position.quote_asset, "USDT");
        assert_eq!(strategy_position.base_asset_balance, base_asset_balance);
        assert_eq!(strategy_position.quote_asset_balance, quote_asset_balance);

        Ok(())
    }
}
