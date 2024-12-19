-- Add down migration script here
DROP TABLE IF EXISTS spot_pairs;
DROP INDEX IF EXISTS idx_spot_pairs_unique;
