-- Add down migration script here
DROP TABLE IF EXISTS strategy_spot_stats;
DROP INDEX IF EXISTS idx_strategy_spot_stats_lookup;
