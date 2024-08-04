CREATE TABLE IF NOT EXISTS coin (
    id VARCHAR PRIMARY KEY,
    name VARCHAR NOT NULL,
    symbol VARCHAR NOT NULL,
    image_uri VARCHAR NOT NULL,
    creator VARCHAR NOT NULL,
    description TEXT NULL,
    twitter VARCHAR NULL,
    telegram VARCHAR NULL,
    website VARCHAR NULL,
    is_listing BOOLEAN NOT NULL DEFAULT FALSE,
    created_at BIGINT NOT NULL,
    create_transaction_hash VARCHAR NOT NULL,
    is_updated BOOLEAN NOT NULL DEFAULT FALSE
);



CREATE INDEX coin_id_index ON coin (id);
CREATE INDEX coin_created_at_index ON coin (created_at);
CREATE INDEX coin_name_index ON coin (name);
CREATE INDEX coin_symbol_index ON coin (symbol);
