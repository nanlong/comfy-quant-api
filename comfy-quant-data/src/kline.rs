use anyhow::Result;
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use sqlx::postgres::PgPool;
use sqlx::FromRow;

#[derive(Debug, Default, FromRow)]
pub struct Kline {
    pub id: i32,
    pub exchange: String,
    pub symbol: String,
    pub interval: String,
    pub open_time: i64,
    pub open_price: Decimal,
    pub high_price: Decimal,
    pub low_price: Decimal,
    pub close_price: Decimal,
    pub volume: Decimal,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

pub async fn insert(pool: &PgPool, kline: &Kline) -> Result<Kline> {
    let kline = sqlx::query_as!(
        Kline,
        r#"
        INSERT INTO klines (exchange, symbol, interval, open_time, open_price, high_price, low_price, close_price, volume, created_at, updated_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, NOW(), NOW())
        RETURNING *
        "#,
        kline.exchange,
        kline.symbol,
        kline.interval,
        kline.open_time,
        kline.open_price,
        kline.high_price,
        kline.low_price,
        kline.close_price,
        kline.volume,
    )
    .fetch_one(pool)
    .await?;

    Ok(kline)
}

pub async fn update(pool: &PgPool, kline: &Kline) -> Result<Kline> {
    let kline = sqlx::query_as!(
        Kline,
        r#"
        UPDATE klines SET high_price = $1, low_price = $2, close_price = $3, volume = $4, updated_at = NOW() WHERE id = $5
        RETURNING *
        "#,
        kline.high_price,
        kline.low_price,
        kline.close_price,
        kline.volume,
        kline.id,
    )
    .fetch_one(pool)
    .await?;

    Ok(kline)
}

pub async fn insert_or_update(pool: &PgPool, kline: &Kline) -> Result<Kline> {
    let kline = if kline.id > 0 {
        update(pool, kline).await?
    } else {
        insert(pool, kline).await?
    };

    println!("Kline: {:?}", kline);

    Ok(kline)
}

pub async fn get_by_id(pool: &PgPool, id: i32) -> Result<Option<Kline>> {
    let kline = sqlx::query_as!(
        Kline,
        r#"
        SELECT * FROM klines WHERE id = $1
        "#,
        id,
    )
    .fetch_optional(pool)
    .await?;

    Ok(kline)
}

pub async fn get_kline(
    pool: &PgPool,
    exchange: &str,
    symbol: &str,
    interval: &str,
    open_time: i64,
) -> Result<Option<Kline>> {
    let kline = sqlx::query_as!(
        Kline,
        r#"
        SELECT * FROM klines WHERE exchange = $1 AND symbol = $2 AND interval = $3 AND open_time = $4
        "#,
        exchange,
        symbol,
        interval,
        open_time,
    )
    .fetch_optional(pool)
    .await?;

    Ok(kline)
}

pub async fn listen_for_kline_changes(pool: &PgPool) -> Result<(), sqlx::Error> {
    sqlx::query("LISTEN kline_change").execute(pool).await?;

    let mut listener = sqlx::postgres::PgListener::connect_with(pool).await?;

    while let Ok(notification) = listener.recv().await {
        let payload = notification.payload();
        let parts: Vec<&str> = payload.split(',').collect();
        if parts.len() == 2 {
            let symbol = parts[0];
            let open_time = parts[1].parse::<i64>().unwrap_or_default();
            println!("Kline 已更改，交易对: {}, 开盘时间: {}", symbol, open_time);
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[sqlx::test(migrator = "crate::MIGRATOR")]
    async fn test_insert_kline(pool: PgPool) -> anyhow::Result<()> {
        let kline = Kline {
            exchange: "binance".to_string(),
            symbol: "BTCUSDT".to_string(),
            interval: "1m".to_string(),
            open_time: 1721817600,
            open_price: "10000".parse::<Decimal>()?,
            high_price: "10000".parse::<Decimal>()?,
            low_price: "10000".parse::<Decimal>()?,
            close_price: "10000".parse::<Decimal>()?,
            volume: "10000".parse::<Decimal>()?,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            ..Default::default()
        };

        let kline = insert(&pool, &kline).await?;

        let kline_inserted = get_by_id(&pool, kline.id).await?.unwrap();

        assert_eq!(kline_inserted.id, 1);
        assert_eq!(kline_inserted.exchange, "binance");
        assert_eq!(kline_inserted.symbol, "BTCUSDT");
        assert_eq!(kline_inserted.interval, "1m");
        assert_eq!(kline_inserted.open_price, "10000".parse::<Decimal>()?);
        assert_eq!(kline_inserted.high_price, "10000".parse::<Decimal>()?);
        assert_eq!(kline_inserted.low_price, "10000".parse::<Decimal>()?);
        assert_eq!(kline_inserted.close_price, "10000".parse::<Decimal>()?);
        assert_eq!(kline_inserted.volume, "10000".parse::<Decimal>()?);

        Ok(())
    }

    #[sqlx::test(migrator = "crate::MIGRATOR")]
    async fn test_update_kline(pool: PgPool) -> anyhow::Result<()> {
        let kline = Kline {
            exchange: "binance".to_string(),
            symbol: "BTCUSDT".to_string(),
            interval: "1m".to_string(),
            open_time: 1721817600,
            open_price: "10000".parse::<Decimal>()?,
            high_price: "10000".parse::<Decimal>()?,
            low_price: "10000".parse::<Decimal>()?,
            close_price: "10000".parse::<Decimal>()?,
            volume: "10000".parse::<Decimal>()?,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            ..Default::default()
        };

        let mut kline = insert(&pool, &kline).await?;

        kline.high_price = "20000".parse::<Decimal>()?;
        kline.low_price = "20000".parse::<Decimal>()?;
        kline.close_price = "20000".parse::<Decimal>()?;
        kline.volume = "20000".parse::<Decimal>()?;

        let kline_updated = update(&pool, &kline).await?;

        assert_eq!(kline_updated.id, 1);
        assert_eq!(kline_updated.high_price, "20000".parse::<Decimal>()?);
        assert_eq!(kline_updated.low_price, "20000".parse::<Decimal>()?);
        assert_eq!(kline_updated.close_price, "20000".parse::<Decimal>()?);
        assert_eq!(kline_updated.volume, "20000".parse::<Decimal>()?);

        Ok(())
    }

    // #[sqlx::test(migrator = "crate::MIGRATOR")]
    // async fn test_listen_for_kline_changes(pool: PgPool) -> anyhow::Result<()> {
    //     let mut listener = sqlx::postgres::PgListener::connect_with(&pool).await?;
    //     listener.listen("kline_change").await?;

    //     let kline = Kline {
    //         exchange: "binance".to_string(),
    //         symbol: "BTCUSDT".to_string(),
    //         interval: "1m".to_string(),
    //         open_time: 1721817600,
    //         open_price: "10000".parse::<Decimal>()?,
    //         high_price: "10000".parse::<Decimal>()?,
    //         low_price: "10000".parse::<Decimal>()?,
    //         close_price: "10000".parse::<Decimal>()?,
    //         volume: "10000".parse::<Decimal>()?,
    //         created_at: Utc::now(),
    //         updated_at: Utc::now(),
    //         ..Default::default()
    //     };

    //     insert(&pool, &kline).await?;

    //     // let kline = insert(&pool, &kline).await?;

    //     let notification = listener.recv().await.unwrap();
    //     println!("Received notification: {}", notification.payload());

    //     assert!(true);

    //     Ok(())
    // }

    #[sqlx::test(migrator = "crate::MIGRATOR")]
    async fn test_get_kline(pool: PgPool) -> anyhow::Result<()> {
        let kline = Kline {
            exchange: "binance".to_string(),
            symbol: "BTCUSDT".to_string(),
            interval: "1m".to_string(),
            open_time: 1721817600,
            open_price: "10000".parse::<Decimal>()?,
            high_price: "10000".parse::<Decimal>()?,
            low_price: "10000".parse::<Decimal>()?,
            close_price: "10000".parse::<Decimal>()?,
            volume: "10000".parse::<Decimal>()?,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            ..Default::default()
        };

        let kline = insert(&pool, &kline).await?;

        let kline_get = get_kline(
            &pool,
            &kline.exchange,
            &kline.symbol,
            &kline.interval,
            kline.open_time,
        )
        .await?
        .ok_or_else(|| anyhow::anyhow!("Kline not found"))?;

        assert_eq!(kline_get.id, kline.id);
        assert_eq!(kline_get.exchange, kline.exchange);
        assert_eq!(kline_get.symbol, kline.symbol);
        assert_eq!(kline_get.interval, kline.interval);
        assert_eq!(kline_get.open_time, kline.open_time);

        Ok(())
    }
}
