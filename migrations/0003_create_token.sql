CREATE TABLE IF NOT EXISTS token (
    id VARCHAR(42) PRIMARY KEY,
    name VARCHAR NOT NULL,
    symbol VARCHAR NOT NULL,
    image_uri VARCHAR NOT NULL,
    creator VARCHAR(42)NOT NULL REFERENCES account(id),
    description TEXT NULL,
    twitter VARCHAR NULL,
    telegram VARCHAR NULL,
    website VARCHAR NULL,
    is_listing BOOLEAN NOT NULL DEFAULT FALSE,
    created_at BIGINT NOT NULL,
    create_transaction_hash VARCHAR NOT NULL,
    is_updated BOOLEAN NOT NULL DEFAULT FALSE,
    pair VARCHAR(42) NULL
);



CREATE INDEX coin_id_index ON token (id);
CREATE INDEX coin_created_at_index ON token (created_at);
CREATE INDEX coin_name_index ON token (name);
CREATE INDEX coin_symbol_index ON token (symbol);
