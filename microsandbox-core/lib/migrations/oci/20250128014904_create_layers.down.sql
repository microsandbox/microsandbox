-- Add down migration script here

DROP INDEX IF EXISTS idx_layers_digest;
DROP INDEX IF EXISTS idx_layers_diff_id;
DROP TABLE IF EXISTS layers;
