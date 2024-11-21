-- Add migration script here
-- 策略持仓信息
DROP INDEX IF EXISTS idx_strategy_positions_lookup;
DROP TABLE IF EXISTS strategy_positions;
