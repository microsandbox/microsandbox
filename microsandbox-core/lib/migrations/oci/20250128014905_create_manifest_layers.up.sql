-- Add up migration script here

-- Create manifest_layers table
CREATE TABLE IF NOT EXISTS manifest_layers (
    id INTEGER PRIMARY KEY,
    manifest_id INTEGER NOT NULL,
    layer_id INTEGER NOT NULL,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    modified_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (manifest_id) REFERENCES manifests(id) ON DELETE CASCADE,
    FOREIGN KEY (layer_id) REFERENCES layers(id) ON DELETE CASCADE,
    UNIQUE(manifest_id, layer_id)
);

-- Create indexes
CREATE INDEX IF NOT EXISTS idx_manifest_layers_manifest_id ON manifest_layers(manifest_id);
CREATE INDEX IF NOT EXISTS idx_manifest_layers_layer_id ON manifest_layers(layer_id);
