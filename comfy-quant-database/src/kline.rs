use anyhow::Result;
use bon::Builder;
use chrono::{DateTime, Utc};
use comfy_quant_base::{Exchange, KlineInterval, Market, Symbol};
use futures::stream::BoxStream;
use rust_decimal::Decimal;
use sqlx::{postgres::PgPool, FromRow};

#[derive(Debug, FromRow)]
pub struct Kline {
    pub id: i32,                   // 主键ID
    pub exchange: Exchange,        // 交易所
    pub market: Market,            // 市场
    pub symbol: Symbol,            // 交易对
    pub interval: KlineInterval,   // 时间间隔
    pub open_time: DateTime<Utc>,  // 开盘时间
    pub open_price: Decimal,       // 开盘价格
    pub high_price: Decimal,       // 最高价格
    pub low_price: Decimal,        // 最低价格
    pub close_price: Decimal,      // 收盘价格
    pub volume: Decimal,           // 成交量
    pub created_at: DateTime<Utc>, // 创建时间
    pub updated_at: DateTime<Utc>, // 更新时间
}

#[derive(Builder)]
#[builder(on(_, into))]
pub struct CreateKlineParams {
    pub exchange: Exchange,       // 交易所
    pub market: Market,           // 市场
    pub symbol: Symbol,           // 交易对
    pub interval: KlineInterval,  // 时间间隔
    pub open_time: DateTime<Utc>, // 开盘时间
    pub open_price: Decimal,      // 开盘价格
    pub high_price: Decimal,      // 最高价格
    pub low_price: Decimal,       // 最低价格
    pub close_price: Decimal,     // 收盘价格
    pub volume: Decimal,          // 成交量
}

#[derive(Builder)]
#[builder(on(_, into))]
pub struct UpdateKlineParams {
    pub id: i32,              // 主键ID
    pub high_price: Decimal,  // 最高价格
    pub low_price: Decimal,   // 最低价格
    pub close_price: Decimal, // 收盘价格
    pub volume: Decimal,      // 成交量
}

pub async fn create(db: &PgPool, data: CreateKlineParams) -> Result<Kline> {
    let kline = sqlx::query_as!(
        Kline,
        r#"
        INSERT INTO klines (exchange, market, symbol, interval, open_time, open_price, high_price, low_price, close_price, volume, created_at, updated_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, NOW(), NOW())
        RETURNING *
        "#,
        data.exchange.as_ref(),
        data.market.as_ref(),
        data.symbol.as_ref(),
        data.interval.as_ref(),
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

pub async fn update(db: &PgPool, data: UpdateKlineParams) -> Result<Kline> {
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

pub async fn create_or_update(db: &PgPool, data: CreateKlineParams) -> Result<Kline> {
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
        data.exchange.as_ref(),
        data.market.as_ref(),
        data.symbol.as_ref(),
        data.interval.as_ref(),
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
    exchange: &Exchange,
    market: &Market,
    symbol: &Symbol,
    interval: &KlineInterval,
    open_time: &DateTime<Utc>,
) -> Result<Option<Kline>> {
    let kline = sqlx::query_as!(
        Kline,
        r#"
        SELECT * FROM klines WHERE exchange = $1 AND market = $2 AND symbol = $3 AND interval = $4 AND open_time = $5
        "#,
        exchange.as_ref(),
        market.as_ref(),
        symbol.as_ref(),
        interval.as_ref(),
        open_time,
    )
    .fetch_optional(db)
    .await?;

    Ok(kline)
}

pub async fn list(
    db: &PgPool,
    exchange: &Exchange,
    market: &Market,
    symbol: &Symbol,
    interval: &KlineInterval,
    start_datetime: &DateTime<Utc>,
    end_datetime: &DateTime<Utc>,
) -> Result<Vec<Kline>> {
    let result = sqlx::query_as!(
        Kline,
        r#"
        SELECT * FROM klines
            WHERE
                exchange = $1 AND
                market = $2 AND
                symbol = $3 AND
                interval = $4 AND
                open_time BETWEEN $5 AND $6
            ORDER BY open_time ASC
        "#,
        exchange.as_ref(),
        market.as_ref(),
        symbol.as_ref(),
        interval.as_ref(),
        start_datetime,
        end_datetime
    )
    .fetch_all(db)
    .await?;

    Ok(result)
}

pub fn time_range_klines_stream<'a>(
    db: &'a PgPool,
    exchange: &Exchange,
    market: &Market,
    symbol: &Symbol,
    interval: &KlineInterval,
    start_datetime: &DateTime<Utc>,
    end_datetime: &DateTime<Utc>,
) -> BoxStream<'a, Result<Kline, sqlx::Error>> {
    sqlx::query_as!(
        Kline,
        r#"
        SELECT * FROM klines WHERE exchange = $1 AND market = $2 AND symbol = $3 AND interval = $4 AND open_time >= $5 AND open_time <= $6 ORDER BY open_time ASC
        "#,
        exchange.as_ref(),
        market.as_ref(),
        symbol.as_ref(),
        interval.as_ref(),
        start_datetime,
        end_datetime,
    )
    .fetch(db)
}

pub async fn time_range_klines_count(
    db: &PgPool,
    exchange: &Exchange,
    market: &Market,
    symbol: &Symbol,
    interval: &KlineInterval,
    start_datetime: &DateTime<Utc>,
    end_datetime: &DateTime<Utc>,
) -> Result<usize> {
    let count = sqlx::query_scalar!(
        r#"
        SELECT COUNT(*) FROM klines WHERE exchange = $1 AND market = $2 AND symbol = $3 AND interval = $4 AND open_time >= $5 AND open_time <= $6
        "#,
        exchange.as_ref(),
        market.as_ref(),
        symbol.as_ref(),
        interval.as_ref(),
        start_datetime,
        end_datetime,
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

#[cfg(test)]
mod tests {
    use comfy_quant_base::secs_to_datetime;
    use futures::StreamExt;
    use rust_decimal_macros::dec;

    use super::*;

    async fn create_kline(db: &PgPool) -> Result<Kline> {
        let open_time = secs_to_datetime(1721817600)?;

        let data = CreateKlineParams::builder()
            .exchange(Exchange::Binance)
            .market(Market::Spot)
            .symbol("BTCUSDT")
            .interval("1m")
            .open_time(open_time)
            .open_price(dec!(10000))
            .high_price(dec!(10000))
            .low_price(dec!(10000))
            .close_price(dec!(10000))
            .volume(dec!(10000))
            .build();

        let kline = create(&db, data).await?;

        Ok(kline)
    }

    #[sqlx::test(migrator = "crate::MIGRATOR")]
    async fn test_create_kline(db: PgPool) -> Result<()> {
        let kline = create_kline(&db).await?;

        let kline_created = get_by_id(&db, kline.id).await?.unwrap();

        assert_eq!(kline_created.id, 1);
        assert_eq!(kline_created.exchange, Exchange::Binance);
        assert_eq!(kline_created.market, Market::Spot);
        assert_eq!(kline_created.symbol, "BTCUSDT".into());
        assert_eq!(kline_created.interval, "1m".into());
        assert_eq!(kline_created.open_price, dec!(10000));
        assert_eq!(kline_created.high_price, dec!(10000));
        assert_eq!(kline_created.low_price, dec!(10000));
        assert_eq!(kline_created.close_price, dec!(10000));
        assert_eq!(kline_created.volume, dec!(10000));

        Ok(())
    }

    #[sqlx::test(migrator = "crate::MIGRATOR")]
    async fn test_update_kline(db: PgPool) -> Result<()> {
        let kline = create_kline(&db).await?;

        let data = UpdateKlineParams::builder()
            .id(kline.id)
            .high_price(dec!(20000))
            .low_price(dec!(20000))
            .close_price(dec!(20000))
            .volume(dec!(20000))
            .build();

        let kline_updated = update(&db, data).await?;

        assert_eq!(kline_updated.id, 1);
        assert_eq!(kline_updated.high_price, dec!(20000));
        assert_eq!(kline_updated.low_price, dec!(20000));
        assert_eq!(kline_updated.close_price, dec!(20000));
        assert_eq!(kline_updated.volume, dec!(20000));

        Ok(())
    }

    #[sqlx::test(migrator = "crate::MIGRATOR")]
    async fn test_create_or_update_kline(db: PgPool) -> Result<()> {
        let open_time = secs_to_datetime(1721817600)?;

        let data = CreateKlineParams::builder()
            .exchange(Exchange::Binance)
            .market(Market::Spot)
            .symbol("BTCUSDT")
            .interval(KlineInterval::OneMinute)
            .open_time(open_time)
            .open_price(dec!(10000))
            .high_price(dec!(10000))
            .low_price(dec!(10000))
            .close_price(dec!(10000))
            .volume(dec!(10000))
            .build();

        let kline = create_or_update(&db, data).await?;

        assert_eq!(kline.id, 1);
        assert_eq!(kline.exchange, Exchange::Binance);
        assert_eq!(kline.market, Market::Spot);
        assert_eq!(kline.symbol, "BTCUSDT".into());
        assert_eq!(kline.interval, KlineInterval::OneMinute);
        assert_eq!(kline.open_time.timestamp(), 1721817600);
        assert_eq!(kline.open_price, dec!(10000));
        assert_eq!(kline.high_price, dec!(10000));
        assert_eq!(kline.low_price, dec!(10000));
        assert_eq!(kline.close_price, dec!(10000));
        assert_eq!(kline.volume, dec!(10000));

        let data2 = CreateKlineParams::builder()
            .exchange(Exchange::Binance)
            .market(Market::Spot)
            .symbol("BTCUSDT")
            .interval(KlineInterval::OneMinute)
            .open_time(open_time)
            .open_price(dec!(20000))
            .high_price(dec!(20000))
            .low_price(dec!(20000))
            .close_price(dec!(20000))
            .volume(dec!(20000))
            .build();

        let kline2 = create_or_update(&db, data2).await?;

        assert_eq!(kline2.id, 1);
        assert_eq!(kline2.open_price, dec!(20000));
        assert_eq!(kline2.high_price, dec!(20000));
        assert_eq!(kline2.low_price, dec!(20000));
        assert_eq!(kline2.close_price, dec!(20000));
        assert_eq!(kline2.volume, dec!(20000));

        Ok(())
    }

    // #[sqlx::test(migrator = "crate::MIGRATOR")]
    // async fn test_listen_for_kline_changes(db: PgPool) -> Result<()> {
    //     let mut listener = sqlx::postgres::PgListener::connect_with(&db).await?;
    //     listener.listen("kline_change").await?;

    //     let kline = Kline {
    //         exchange: "binance".to_string(),
    //         symbol: "BTCUSDT".to_string(),
    //         interval: "1m".to_string(),
    //         open_time: 1721817600,
    //         open_price: dec!(10000),
    //         high_price: dec!(10000),
    //         low_price: dec!(10000),
    //         close_price: dec!(10000),
    //         volume: dec!(10000),
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
    async fn test_get_kline(db: PgPool) -> Result<()> {
        let kline = create_kline(&db).await?;

        let kline_get = get_kline(
            &db,
            &kline.exchange,
            &kline.market,
            &kline.symbol,
            &kline.interval,
            &kline.open_time,
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
    async fn test_time_range_klines_stream(db: PgPool) -> Result<()> {
        create_kline(&db).await?;
        let start_datetime = secs_to_datetime(1721817600)?;
        let end_datetime = secs_to_datetime(1721817600)?;

        let klines = time_range_klines_stream(
            &db,
            &Exchange::Binance,
            &Market::Spot,
            &"BTCUSDT".into(),
            &"1m".into(),
            &start_datetime,
            &end_datetime,
        );

        let klines = klines.collect::<Vec<Result<Kline, sqlx::Error>>>().await;

        assert_eq!(klines.len(), 1);
        assert_eq!(klines[0].as_ref().unwrap().id, 1_i32);

        Ok(())
    }

    #[sqlx::test(migrator = "crate::MIGRATOR")]
    async fn test_time_range_klines_count(db: PgPool) -> Result<()> {
        create_kline(&db).await?;

        let start_datetime = secs_to_datetime(1721817600)?;
        let end_datetime = secs_to_datetime(1721817600)?;

        let count = time_range_klines_count(
            &db,
            &Exchange::Binance,
            &Market::Spot,
            &"BTCUSDT".into(),
            &"1m".into(),
            &start_datetime,
            &end_datetime,
        )
        .await?;
        assert_eq!(count, 1);

        Ok(())
    }
}
