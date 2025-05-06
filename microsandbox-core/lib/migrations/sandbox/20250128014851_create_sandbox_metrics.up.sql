-- Add up migration script here

-- Create sandbox_metrics table
CREATE TABLE IF NOT EXISTS sandbox_metrics (
    id INTEGER PRIMARY KEY,
    sandbox_id INTEGER NOT NULL,
    timestamp DATETIME DEFAULT CURRENT_TIMESTAMP,
    cpu_usage_percent REAL,
    memory_usage_bytes INTEGER,
    disk_usage_bytes INTEGER,
    FOREIGN KEY(sandbox_id) REFERENCES sandbox(id)
);

-- Create index
CREATE INDEX IF NOT EXISTS idx_sandbox_metrics_sandbox_id_timestamp ON sandbox_metrics(sandbox_id, timestamp);
