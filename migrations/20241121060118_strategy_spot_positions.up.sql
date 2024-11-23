-- Add migration script here
-- 策略持仓信息
CREATE TABLE IF NOT EXISTS strategy_spot_positions (
    id SERIAL PRIMARY KEY,
    workflow_id VARCHAR(21) NOT NULL,
    node_id SMALLINT NOT NULL,
    node_name VARCHAR(20) NOT NULL,
    exchange VARCHAR(20) NOT NULL,
    symbol VARCHAR(20) NOT NULL,
    base_asset VARCHAR(20) NOT NULL,
    quote_asset VARCHAR(20) NOT NULL,
    base_asset_balance NUMERIC NOT NULL,
    quote_asset_balance NUMERIC NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

-- 创建索引（缩短索引名称）
CREATE INDEX IF NOT EXISTS idx_strategy_spot_positions_lookup
ON strategy_spot_positions (workflow_id, node_id, node_name, exchange, symbol, base_asset, quote_asset);

-- 添加表注释
COMMENT ON TABLE strategy_spot_positions IS '策略持仓信息';

-- 添加字段注释
COMMENT ON COLUMN strategy_spot_positions.id IS 'ID';
COMMENT ON COLUMN strategy_spot_positions.workflow_id IS '工作流ID';
COMMENT ON COLUMN strategy_spot_positions.node_id IS '策略节点ID';
COMMENT ON COLUMN strategy_spot_positions.node_name IS '策略节点名称';
COMMENT ON COLUMN strategy_spot_positions.exchange IS '交易所';
COMMENT ON COLUMN strategy_spot_positions.symbol IS '交易对';
COMMENT ON COLUMN strategy_spot_positions.base_asset IS '基础资产';
COMMENT ON COLUMN strategy_spot_positions.quote_asset IS '计价资产';
COMMENT ON COLUMN strategy_spot_positions.base_asset_balance IS '基础资产余额';
COMMENT ON COLUMN strategy_spot_positions.quote_asset_balance IS '计价资产余额';
COMMENT ON COLUMN strategy_spot_positions.created_at IS '创建时间';
