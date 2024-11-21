-- Add migration script here
-- 历史K线数据
DROP INDEX IF EXISTS idx_klines_unique;
DROP TABLE IF EXISTS klines;
