-- 创建K线数据表
CREATE TABLE IF NOT EXISTS klines (
    id SERIAL PRIMARY KEY,
    exchange VARCHAR(20) NOT NULL,
    market VARCHAR(20) NOT NULL,
    symbol VARCHAR(20) NOT NULL,
    interval VARCHAR(10) NOT NULL,
    open_time TIMESTAMPTZ NOT NULL,
    open_price NUMERIC(20,8) NOT NULL,
    high_price NUMERIC(20,8) NOT NULL,
    low_price NUMERIC(20,8) NOT NULL,
    close_price NUMERIC(20,8) NOT NULL,
    volume NUMERIC(30,8) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- 创建唯一索引（缩短名称）
CREATE UNIQUE INDEX IF NOT EXISTS idx_klines_unique
ON klines (exchange, market, symbol, interval, open_time);

-- 添加表注释
COMMENT ON TABLE klines IS '历史K线数据';

-- 添加字段注释
COMMENT ON COLUMN klines.id IS 'ID';
COMMENT ON COLUMN klines.exchange IS '交易所';
COMMENT ON COLUMN klines.market IS '市场';
COMMENT ON COLUMN klines.symbol IS '交易对';
COMMENT ON COLUMN klines.interval IS '时间间隔';
COMMENT ON COLUMN klines.open_time IS '开盘时间';
COMMENT ON COLUMN klines.open_price IS '开盘价格';
COMMENT ON COLUMN klines.high_price IS '最高价格';
COMMENT ON COLUMN klines.low_price IS '最低价格';
COMMENT ON COLUMN klines.close_price IS '收盘价格';
COMMENT ON COLUMN klines.volume IS '成交量';
COMMENT ON COLUMN klines.created_at IS '创建时间';
COMMENT ON COLUMN klines.updated_at IS '更新时间';
