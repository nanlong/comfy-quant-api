-- Add migration script here
-- 历史K线数据
DROP TABLE IF EXISTS klines;
DROP INDEX IF EXISTS idx_klines_unique;
