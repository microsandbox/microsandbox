-- Add up migration script here

-- Create layers table
CREATE TABLE IF NOT EXISTS layers (
    id INTEGER PRIMARY KEY,
    media_type TEXT NOT NULL,
    digest TEXT NOT NULL UNIQUE, -- the hash of the compressed layer
    diff_id TEXT NOT NULL, -- the hash of the uncompressed layer
    size_bytes INTEGER NOT NULL,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    modified_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- Create indexes
CREATE INDEX IF NOT EXISTS idx_layers_digest ON layers(digest);
CREATE INDEX IF NOT EXISTS idx_layers_diff_id ON layers(diff_id);
