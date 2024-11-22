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
    pub market: String,               // 市场
    pub symbol: String,               // 交易对
    pub base_asset: String,           // 基础资产
    pub quote_asset: String,          // 计价资产
    pub base_asset_balance: Decimal,  // 基础资产持仓量
    pub quote_asset_balance: Decimal, // 计价资产持仓量
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
        market: String,
        symbol: String,
        base_asset: String,
        quote_asset: String,
        base_asset_balance: Decimal,
        quote_asset_balance: Decimal,
    ) -> Self {
        StrategySpotPosition {
            workflow_id,
            node_id,
            node_name,
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

pub async fn create(db: &PgPool, data: &StrategySpotPosition) -> Result<StrategySpotPosition> {
    let strategy_spot_position = sqlx::query_as!(
        StrategySpotPosition,
        r#"
        INSERT INTO strategy_spot_positions (workflow_id, node_id, node_name, exchange, market, symbol, base_asset, quote_asset, base_asset_balance, quote_asset_balance, created_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, NOW())
        RETURNING *
        "#,
        data.workflow_id,
        data.node_id,
        data.node_name,
        data.exchange,
        data.market,
        data.symbol,
        data.base_asset,
        data.quote_asset,
        data.base_asset_balance,
        data.quote_asset_balance,
    )
    .fetch_one(db)
    .await?;

    Ok(strategy_spot_position)
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
            .market("spot")
            .symbol("BTCUSDT")
            .base_asset("BTC")
            .quote_asset("USDT")
            .base_asset_balance("1".parse::<Decimal>()?)
            .quote_asset_balance("1000".parse::<Decimal>()?)
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
        assert_eq!(strategy_spot_position.market, "spot");
        assert_eq!(strategy_spot_position.symbol, "BTCUSDT");
        assert_eq!(strategy_spot_position.base_asset, "BTC");
        assert_eq!(strategy_spot_position.quote_asset, "USDT");
        assert_eq!(
            strategy_spot_position.base_asset_balance,
            "1".parse::<Decimal>()?
        );
        assert_eq!(
            strategy_spot_position.quote_asset_balance,
            "1000".parse::<Decimal>()?
        );

        Ok(())
    }
}
