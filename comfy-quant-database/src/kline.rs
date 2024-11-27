use anyhow::Result;
use bon::bon;
use chrono::{DateTime, Utc};
use futures::Stream;
use rust_decimal::Decimal;
use sqlx::{postgres::PgPool, FromRow};
use strum_macros::{AsRefStr, EnumIter, EnumString};

#[derive(Debug, Default, FromRow)]
pub struct Kline {
    pub id: i32,                   // 主键ID
    pub exchange: String,          // 交易所
    pub market: String,            // 市场
    pub symbol: String,            // 交易对
    pub interval: String,          // 时间间隔
    pub open_time: i64,            // 开盘时间
    pub open_price: Decimal,       // 开盘价格
    pub high_price: Decimal,       // 最高价格
    pub low_price: Decimal,        // 最低价格
    pub close_price: Decimal,      // 收盘价格
    pub volume: Decimal,           // 成交量
    pub created_at: DateTime<Utc>, // 创建时间
    pub updated_at: DateTime<Utc>, // 更新时间
}

#[bon]
impl Kline {
    #[builder]
    pub fn new(
        exchange: String,
        market: String,
        symbol: String,
        interval: String,
        open_time: i64,
        open_price: Decimal,
        high_price: Decimal,
        low_price: Decimal,
        close_price: Decimal,
        volume: Decimal,
    ) -> Self {
        Kline {
            exchange,
            market,
            symbol,
            interval,
            open_time,
            open_price,
            high_price,
            low_price,
            close_price,
            volume,
            ..Default::default()
        }
    }
}

pub async fn create(db: &PgPool, data: &Kline) -> Result<Kline> {
    let kline = sqlx::query_as!(
        Kline,
        r#"
        INSERT INTO klines (exchange, market, symbol, interval, open_time, open_price, high_price, low_price, close_price, volume, created_at, updated_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, NOW(), NOW())
        RETURNING *
        "#,
        data.exchange,
        data.market,
        data.symbol,
        data.interval,
        data.open_time,
        data.open_price,
        data.high_price,
        data.low_price,
        data.close_price,
        data.volume,
    )
    .fetch_one(db)
    .await?;

    Ok(kline)
}

pub async fn update(db: &PgPool, data: &Kline) -> Result<Kline> {
    let kline = sqlx::query_as!(
        Kline,
        r#"
        UPDATE klines SET high_price = $1, low_price = $2, close_price = $3, volume = $4, updated_at = NOW() WHERE id = $5
        RETURNING *
        "#,
        data.high_price,
        data.low_price,
        data.close_price,
        data.volume,
        data.id,
    )
    .fetch_one(db)
    .await?;

    Ok(kline)
}

pub async fn create_or_update(db: &PgPool, data: &Kline) -> Result<Kline> {
    let kline = sqlx::query_as!(
        Kline,
        r#"
        INSERT INTO klines (exchange, market, symbol, interval, open_time, open_price, high_price, low_price, close_price, volume, created_at, updated_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, NOW(), NOW())
        ON CONFLICT (exchange, market, symbol, interval, open_time)
        DO UPDATE SET
            open_price = EXCLUDED.open_price,
            high_price = EXCLUDED.high_price,
            low_price = EXCLUDED.low_price,
            close_price = EXCLUDED.close_price,
            volume = EXCLUDED.volume,
            updated_at = NOW()
        RETURNING *
        "#,
        data.exchange,
        data.market,
        data.symbol,
        data.interval,
        data.open_time,
        data.open_price,
        data.high_price,
        data.low_price,
        data.close_price,
        data.volume,
    ).fetch_one(db)
    .await?;

    Ok(kline)
}

pub async fn get_by_id(db: &PgPool, id: i32) -> Result<Option<Kline>> {
    let kline = sqlx::query_as!(
        Kline,
        r#"
        SELECT * FROM klines WHERE id = $1
        "#,
        id,
    )
    .fetch_optional(db)
    .await?;

    Ok(kline)
}

pub async fn get_kline(
    db: &PgPool,
    exchange: &str,
    market: &str,
    symbol: &str,
    interval: &str,
    open_time: i64,
) -> Result<Option<Kline>> {
    let kline = sqlx::query_as!(
        Kline,
        r#"
        SELECT * FROM klines WHERE exchange = $1 AND market = $2 AND symbol = $3 AND interval = $4 AND open_time = $5
        "#,
        exchange,
        market,
        symbol,
        interval,
        open_time,
    )
    .fetch_optional(db)
    .await?;

    Ok(kline)
}

pub fn time_range_klines_stream<'a>(
    db: &'a PgPool,
    exchange: &str,
    market: &str,
    symbol: &str,
    interval: &str,
    start_timestamp_millis: i64,
    end_timestamp_millis: i64,
) -> impl Stream<Item = Result<Kline, sqlx::Error>> + 'a {
    sqlx::query_as!(
        Kline,
        r#"
        SELECT * FROM klines WHERE exchange = $1 AND market = $2 AND symbol = $3 AND interval = $4 AND open_time >= $5 AND open_time <= $6 ORDER BY open_time ASC
        "#,
        exchange,
        market,
        symbol,
        interval,
        start_timestamp_millis,
        end_timestamp_millis,
    )
    .fetch(db)
}

pub async fn time_range_klines_count(
    db: &PgPool,
    exchange: &str,
    market: &str,
    symbol: &str,
    interval: &str,
    start_timestamp_millis: i64,
    end_timestamp_millis: i64,
) -> Result<usize> {
    let count = sqlx::query_scalar!(
        r#"
        SELECT COUNT(*) FROM klines WHERE exchange = $1 AND market = $2 AND symbol = $3 AND interval = $4 AND open_time >= $5 AND open_time <= $6
        "#,
        exchange,
        market,
        symbol,
        interval,
        start_timestamp_millis,
        end_timestamp_millis,
    )
    .fetch_one(db)
    .await?;

    Ok(count.unwrap_or(0) as usize)
}

// pub async fn listen_for_kline_changes(db: &PgPool) -> Result<(), sqlx::Error> {
//     sqlx::query("LISTEN kline_change").execute(db).await?;

//     let mut listener = sqlx::postgres::PgListener::connect_with(db).await?;

//     while let Ok(notification) = listener.recv().await {
//         let payload = notification.payload();
//         let parts: Vec<&str> = payload.split(',').collect();
//         if parts.len() == 2 {
//             let symbol = parts[0];
//             let open_time = parts[1].parse::<i64>().unwrap_or_default();
//             println!("Kline 已更改，交易对: {}, 开盘时间: {}", symbol, open_time);
//         }
//     }

//     Ok(())
// }

#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumString, AsRefStr, EnumIter)]
pub enum KlineInterval {
    #[strum(serialize = "1s")]
    OneSecond,
    #[strum(serialize = "1m")]
    OneMinute,
    #[strum(serialize = "3m")]
    ThreeMinutes,
    #[strum(serialize = "5m")]
    FiveMinutes,
    #[strum(serialize = "15m")]
    FifteenMinutes,
    #[strum(serialize = "30m")]
    ThirtyMinutes,
    #[strum(serialize = "1h")]
    OneHour,
    #[strum(serialize = "2h")]
    TwoHours,
    #[strum(serialize = "4h")]
    FourHours,
    #[strum(serialize = "6h")]
    SixHours,
    #[strum(serialize = "8h")]
    EightHours,
    #[strum(serialize = "12h")]
    TwelveHours,
    #[strum(serialize = "1d")]
    OneDay,
    #[strum(serialize = "3d")]
    ThreeDays,
    #[strum(serialize = "1w")]
    OneWeek,
    #[strum(serialize = "1M")]
    OneMonth,
}

#[cfg(test)]
mod tests {
    use futures::StreamExt;

    use super::*;

    async fn create_kline(db: &PgPool) -> Result<Kline> {
        let kline = Kline::builder()
            .exchange("binance".to_string())
            .market("spot".to_string())
            .symbol("BTCUSDT".to_string())
            .interval("1m".to_string())
            .open_time(1721817600)
            .open_price("10000".parse::<Decimal>()?)
            .high_price("10000".parse::<Decimal>()?)
            .low_price("10000".parse::<Decimal>()?)
            .close_price("10000".parse::<Decimal>()?)
            .volume("10000".parse::<Decimal>()?)
            .build();

        let kline = create(&db, &kline).await?;

        Ok(kline)
    }

    #[test]
    fn test_kline_interval() {
        let interval = KlineInterval::OneMinute;
        assert_eq!(interval.as_ref(), "1m");

        let interval2 = "1m".parse::<KlineInterval>().unwrap();
        assert_eq!(interval2, KlineInterval::OneMinute);

        let interval3 = "1s".parse::<KlineInterval>().unwrap();
        assert_eq!(interval3, KlineInterval::OneSecond);

        let err = "1x".parse::<KlineInterval>().unwrap_err();
        assert_eq!(err.to_string(), "Matching variant not found");
    }

    #[sqlx::test(migrator = "crate::MIGRATOR")]
    async fn test_create_kline(db: PgPool) -> anyhow::Result<()> {
        let kline = create_kline(&db).await?;

        let kline_createed = get_by_id(&db, kline.id).await?.unwrap();

        assert_eq!(kline_createed.id, 1);
        assert_eq!(kline_createed.exchange, "binance");
        assert_eq!(kline_createed.market, "spot");
        assert_eq!(kline_createed.symbol, "BTCUSDT");
        assert_eq!(kline_createed.interval, "1m");
        assert_eq!(kline_createed.open_price, "10000".parse::<Decimal>()?);
        assert_eq!(kline_createed.high_price, "10000".parse::<Decimal>()?);
        assert_eq!(kline_createed.low_price, "10000".parse::<Decimal>()?);
        assert_eq!(kline_createed.close_price, "10000".parse::<Decimal>()?);
        assert_eq!(kline_createed.volume, "10000".parse::<Decimal>()?);

        Ok(())
    }

    #[sqlx::test(migrator = "crate::MIGRATOR")]
    async fn test_update_kline(db: PgPool) -> anyhow::Result<()> {
        let mut kline = create_kline(&db).await?;

        kline.high_price = "20000".parse::<Decimal>()?;
        kline.low_price = "20000".parse::<Decimal>()?;
        kline.close_price = "20000".parse::<Decimal>()?;
        kline.volume = "20000".parse::<Decimal>()?;

        let kline_updated = update(&db, &kline).await?;

        assert_eq!(kline_updated.id, 1);
        assert_eq!(kline_updated.high_price, "20000".parse::<Decimal>()?);
        assert_eq!(kline_updated.low_price, "20000".parse::<Decimal>()?);
        assert_eq!(kline_updated.close_price, "20000".parse::<Decimal>()?);
        assert_eq!(kline_updated.volume, "20000".parse::<Decimal>()?);

        Ok(())
    }

    #[sqlx::test(migrator = "crate::MIGRATOR")]
    async fn test_create_or_update_kline(db: PgPool) -> anyhow::Result<()> {
        let kline = Kline::builder()
            .exchange("binance".to_string())
            .market("spot".to_string())
            .symbol("BTCUSDT".to_string())
            .interval("1m".to_string())
            .open_time(1721817600)
            .open_price("10000".parse::<Decimal>()?)
            .high_price("10000".parse::<Decimal>()?)
            .low_price("10000".parse::<Decimal>()?)
            .close_price("10000".parse::<Decimal>()?)
            .volume("10000".parse::<Decimal>()?)
            .build();

        let kline = create_or_update(&db, &kline).await?;

        assert_eq!(kline.id, 1);
        assert_eq!(kline.exchange, "binance");
        assert_eq!(kline.market, "spot");
        assert_eq!(kline.symbol, "BTCUSDT");
        assert_eq!(kline.interval, "1m");
        assert_eq!(kline.open_time, 1721817600);
        assert_eq!(kline.open_price, "10000".parse::<Decimal>()?);
        assert_eq!(kline.high_price, "10000".parse::<Decimal>()?);
        assert_eq!(kline.low_price, "10000".parse::<Decimal>()?);
        assert_eq!(kline.close_price, "10000".parse::<Decimal>()?);
        assert_eq!(kline.volume, "10000".parse::<Decimal>()?);

        let kline2 = Kline::builder()
            .exchange("binance".to_string())
            .market("spot".to_string())
            .symbol("BTCUSDT".to_string())
            .interval("1m".to_string())
            .open_time(1721817600)
            .open_price("20000".parse::<Decimal>()?)
            .high_price("20000".parse::<Decimal>()?)
            .low_price("20000".parse::<Decimal>()?)
            .close_price("20000".parse::<Decimal>()?)
            .volume("20000".parse::<Decimal>()?)
            .build();

        let kline2 = create_or_update(&db, &kline2).await?;

        assert_eq!(kline2.id, 1);
        assert_eq!(kline2.open_price, "20000".parse::<Decimal>()?);
        assert_eq!(kline2.high_price, "20000".parse::<Decimal>()?);
        assert_eq!(kline2.low_price, "20000".parse::<Decimal>()?);
        assert_eq!(kline2.close_price, "20000".parse::<Decimal>()?);
        assert_eq!(kline2.volume, "20000".parse::<Decimal>()?);

        Ok(())
    }

    // #[sqlx::test(migrator = "crate::MIGRATOR")]
    // async fn test_listen_for_kline_changes(db: PgPool) -> anyhow::Result<()> {
    //     let mut listener = sqlx::postgres::PgListener::connect_with(&db).await?;
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

    //     create(&db, &kline).await?;

    //     // let kline = create(&db, &kline).await?;

    //     let notification = listener.recv().await.unwrap();
    //     println!("Received notification: {}", notification.payload());

    //     assert!(true);

    //     Ok(())
    // }

    #[sqlx::test(migrator = "crate::MIGRATOR")]
    async fn test_get_kline(db: PgPool) -> anyhow::Result<()> {
        let kline = create_kline(&db).await?;

        let kline_get = get_kline(
            &db,
            &kline.exchange,
            &kline.market,
            &kline.symbol,
            &kline.interval,
            kline.open_time,
        )
        .await?
        .ok_or_else(|| anyhow::anyhow!("Kline not found"))?;

        assert_eq!(kline_get.id, kline.id);
        assert_eq!(kline_get.exchange, kline.exchange);
        assert_eq!(kline_get.market, kline.market);
        assert_eq!(kline_get.symbol, kline.symbol);
        assert_eq!(kline_get.interval, kline.interval);
        assert_eq!(kline_get.open_time, kline.open_time);

        Ok(())
    }

    #[sqlx::test(migrator = "crate::MIGRATOR")]
    async fn test_time_range_klines_stream(db: PgPool) -> anyhow::Result<()> {
        create_kline(&db).await?;

        let klines = time_range_klines_stream(
            &db, "binance", "spot", "BTCUSDT", "1m", 1721817600, 1721817600,
        );

        let klines = klines.collect::<Vec<Result<Kline, sqlx::Error>>>().await;

        assert_eq!(klines.len(), 1);
        assert_eq!(klines[0].as_ref().unwrap().id, 1_i32);

        Ok(())
    }

    #[sqlx::test(migrator = "crate::MIGRATOR")]
    async fn test_time_range_klines_count(db: PgPool) -> anyhow::Result<()> {
        create_kline(&db).await?;

        let count = time_range_klines_count(
            &db, "binance", "spot", "BTCUSDT", "1m", 1721817600, 1721817600,
        )
        .await?;
        assert_eq!(count, 1);

        Ok(())
    }
}
