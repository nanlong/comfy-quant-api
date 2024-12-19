-- Add up migration script here
CREATE TABLE IF NOT EXISTS spot_pairs (
    id SERIAL PRIMARY KEY,
    exchange VARCHAR(20) NOT NULL,
    symbol VARCHAR(20) NOT NULL,
    base_asset VARCHAR(20) NOT NULL,
    quote_asset VARCHAR(20) NOT NULL,
    base_asset_precision INT NOT NULL,
    quote_asset_precision INT NOT NULL,
    quote_precision INT NOT NULL,
    status VARCHAR(20) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- 创建索引（缩短索引名称）
CREATE UNIQUE INDEX IF NOT EXISTS idx_spot_pairs_unique
ON spot_pairs (exchange, symbol);

-- 添加表注释
COMMENT ON TABLE spot_pairs IS '现货交易对';

-- 添加字段注释
COMMENT ON COLUMN spot_pairs.id IS 'ID';
COMMENT ON COLUMN spot_pairs.exchange IS '交易所';
COMMENT ON COLUMN spot_pairs.symbol IS '交易对';
COMMENT ON COLUMN spot_pairs.base_asset IS '基础资产';
COMMENT ON COLUMN spot_pairs.quote_asset IS '计价资产';
COMMENT ON COLUMN spot_pairs.base_asset_precision IS '基础资产精度';
COMMENT ON COLUMN spot_pairs.quote_asset_precision IS '计价资产精度';
COMMENT ON COLUMN spot_pairs.quote_precision IS '计价精度';
COMMENT ON COLUMN spot_pairs.status IS '状态';
COMMENT ON COLUMN spot_pairs.created_at IS '创建时间';
COMMENT ON COLUMN spot_pairs.updated_at IS '更新时间';
