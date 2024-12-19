use anyhow::Result;
use chrono::{DateTime, Utc};
use comfy_quant_base::{Exchange, Symbol};
use sqlx::{FromRow, PgPool};

#[derive(Debug, FromRow)]
pub struct SpotPair {
    pub id: i32,                    // 主键ID
    pub exchange: Exchange,         // 交易所
    pub symbol: Symbol,             // 交易对
    pub base_asset: String,         // 基础资产
    pub quote_asset: String,        // 计价资产
    pub base_asset_precision: i32,  // 基础资产精度
    pub quote_asset_precision: i32, // 计价资产精度
    pub quote_precision: i32,       // 计价精度
    pub status: String,             // 状态
    pub created_at: DateTime<Utc>,  // 创建时间
    pub updated_at: DateTime<Utc>,  // 更新时间
}

pub struct CreateSpotPairParams {
    pub exchange: Exchange,         // 交易所
    pub symbol: Symbol,             // 交易对
    pub base_asset: String,         // 基础资产
    pub quote_asset: String,        // 计价资产
    pub base_asset_precision: i32,  // 基础资产精度
    pub quote_asset_precision: i32, // 计价资产精度
    pub quote_precision: i32,       // 计价精度
    pub status: String,             // 状态
}

pub async fn create_or_update(db: &PgPool, data: CreateSpotPairParams) -> Result<SpotPair> {
    let row = sqlx::query_as!(
        SpotPair,
        r#"
        INSERT INTO spot_pairs (exchange, symbol, base_asset, quote_asset, base_asset_precision, quote_asset_precision, quote_precision, status, created_at, updated_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, NOW(), NOW())
        ON CONFLICT (exchange, symbol)
        DO UPDATE SET
            status = EXCLUDED.status,
            updated_at = NOW()
        RETURNING *
        "#,
        data.exchange.as_ref(),
        data.symbol.as_ref(),
        data.base_asset,
        data.quote_asset,
        data.base_asset_precision,
        data.quote_asset_precision,
        data.quote_precision,
        data.status,
    )
    .fetch_one(db)
    .await?;

    Ok(row)
}

pub async fn get(db: &PgPool, exchange: &Exchange, symbol: &Symbol) -> Result<SpotPair> {
    let row = sqlx::query_as!(
        SpotPair,
        r#"
        SELECT * FROM spot_pairs where exchange = $1 AND symbol = $2
        "#,
        exchange.as_ref(),
        symbol.as_ref(),
    )
    .fetch_one(db)
    .await?;

    Ok(row)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[sqlx::test(migrator = "crate::MIGRATOR")]
    async fn test_spot_pairs_create_or_update_should_work(db: PgPool) -> Result<()> {
        let data = CreateSpotPairParams {
            exchange: Exchange::Binance,
            symbol: Symbol::new("BTCUSDT"),
            base_asset: "BTC".into(),
            quote_asset: "USDT".into(),
            base_asset_precision: 8,
            quote_asset_precision: 8,
            quote_precision: 8,
            status: "TRADING".into(),
        };

        let spot_pair = create_or_update(&db, data).await?;

        assert_eq!(spot_pair.id, 1);
        assert_eq!(spot_pair.exchange, Exchange::Binance);
        assert_eq!(spot_pair.symbol, Symbol::new("BTCUSDT"));
        assert_eq!(spot_pair.base_asset, "BTC".to_string());
        assert_eq!(spot_pair.quote_asset, "USDT".to_string());
        assert_eq!(spot_pair.base_asset_precision, 8);
        assert_eq!(spot_pair.quote_asset_precision, 8);
        assert_eq!(spot_pair.quote_precision, 8);
        assert_eq!(spot_pair.status, "TRADING".to_string());

        let data = CreateSpotPairParams {
            exchange: Exchange::Binance,
            symbol: Symbol::new("BTCUSDT"),
            base_asset: "BTC".into(),
            quote_asset: "USDT".into(),
            base_asset_precision: 8,
            quote_asset_precision: 8,
            quote_precision: 8,
            status: "STOP".into(),
        };

        let spot_pair = create_or_update(&db, data).await?;

        assert_eq!(spot_pair.status, "STOP".to_string());

        Ok(())
    }

    #[sqlx::test(migrator = "crate::MIGRATOR")]
    async fn test_spot_pairs_get_should_work(db: PgPool) -> Result<()> {
        let data = CreateSpotPairParams {
            exchange: Exchange::Binance,
            symbol: Symbol::new("BTCUSDT"),
            base_asset: "BTC".into(),
            quote_asset: "USDT".into(),
            base_asset_precision: 8,
            quote_asset_precision: 8,
            quote_precision: 8,
            status: "TRADING".into(),
        };

        create_or_update(&db, data).await?;

        let spot_pair = get(&db, &Exchange::Binance, &Symbol::new("BTCUSDT")).await?;

        assert_eq!(spot_pair.id, 1);
        assert_eq!(spot_pair.exchange, Exchange::Binance);
        assert_eq!(spot_pair.symbol, Symbol::new("BTCUSDT"));

        Ok(())
    }
}
