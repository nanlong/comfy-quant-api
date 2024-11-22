-- Add migration script here
-- 策略持仓信息
DROP TABLE IF EXISTS strategy_spot_positions;
DROP INDEX IF EXISTS idx_strategy_spot_positions_lookup;
