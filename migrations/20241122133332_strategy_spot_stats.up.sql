-- Add up migration script here
-- 策略统计信息
CREATE TABLE IF NOT EXISTS strategy_spot_stats (
    id SERIAL PRIMARY KEY,
    workflow_id VARCHAR(21) NOT NULL,
    node_id SMALLINT NOT NULL,
    node_name VARCHAR(20) NOT NULL,
    exchange VARCHAR(20) NOT NULL,
    symbol VARCHAR(20) NOT NULL,
    base_asset VARCHAR(20) NOT NULL,
    quote_asset VARCHAR(20) NOT NULL,
    initial_base_balance NUMERIC NOT NULL,
    initial_quote_balance NUMERIC NOT NULL,
    maker_commission_rate NUMERIC NOT NULL,
    taker_commission_rate NUMERIC NOT NULL,
    base_asset_balance NUMERIC NOT NULL,
    quote_asset_balance NUMERIC NOT NULL,
    avg_price NUMERIC NOT NULL,
    total_trades BIGINT NOT NULL,
    buy_trades BIGINT NOT NULL,
    sell_trades BIGINT NOT NULL,
    total_base_volume NUMERIC NOT NULL,
    total_quote_volume NUMERIC NOT NULL,
    total_base_commission NUMERIC NOT NULL,
    total_quote_commission NUMERIC NOT NULL,
    realized_pnl NUMERIC NOT NULL,
    win_trades BIGINT NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

-- 创建索引（缩短索引名称）
CREATE UNIQUE INDEX IF NOT EXISTS idx_strategy_spot_stats_unique
ON strategy_spot_stats (workflow_id, node_id, node_name, exchange, symbol, base_asset, quote_asset);

-- 添加表注释
COMMENT ON TABLE strategy_spot_stats IS '策略统计信息';

-- 添加字段注释
COMMENT ON COLUMN strategy_spot_stats.id IS 'ID';
COMMENT ON COLUMN strategy_spot_stats.workflow_id IS '工作流ID';
COMMENT ON COLUMN strategy_spot_stats.node_id IS '策略节点ID';
COMMENT ON COLUMN strategy_spot_stats.node_name IS '策略节点名称';
COMMENT ON COLUMN strategy_spot_stats.exchange IS '交易所';
COMMENT ON COLUMN strategy_spot_stats.symbol IS '交易对';
COMMENT ON COLUMN strategy_spot_stats.base_asset IS '基础资产';
COMMENT ON COLUMN strategy_spot_stats.quote_asset IS '计价资产';
COMMENT ON COLUMN strategy_spot_stats.initial_base_balance IS '初始化基础资产余额';
COMMENT ON COLUMN strategy_spot_stats.initial_quote_balance IS '初始化计价资产余额';
COMMENT ON COLUMN strategy_spot_stats.maker_commission_rate IS 'maker手续费率';
COMMENT ON COLUMN strategy_spot_stats.taker_commission_rate IS 'taker手续费率';
COMMENT ON COLUMN strategy_spot_stats.base_asset_balance IS '基础资产持仓量';
COMMENT ON COLUMN strategy_spot_stats.quote_asset_balance IS '计价资产持仓量';
COMMENT ON COLUMN strategy_spot_stats.avg_price IS '基础资产持仓均价';
COMMENT ON COLUMN strategy_spot_stats.total_trades IS '总交易次数';
COMMENT ON COLUMN strategy_spot_stats.buy_trades IS '买入次数';
COMMENT ON COLUMN strategy_spot_stats.sell_trades IS '卖出次数';
COMMENT ON COLUMN strategy_spot_stats.total_base_volume IS '基础资产交易量';
COMMENT ON COLUMN strategy_spot_stats.total_quote_volume IS '计价资产交易量';
COMMENT ON COLUMN strategy_spot_stats.total_base_commission IS '基础资产总手续费';
COMMENT ON COLUMN strategy_spot_stats.total_quote_commission IS '计价资产总手续费';
COMMENT ON COLUMN strategy_spot_stats.realized_pnl IS '已实现盈亏';
COMMENT ON COLUMN strategy_spot_stats.win_trades IS '盈利交易次数';
COMMENT ON COLUMN strategy_spot_stats.created_at IS '创建时间';
COMMENT ON COLUMN strategy_spot_stats.updated_at IS '更新时间';
