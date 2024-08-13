CREATE TABLE IF NOT EXISTS king (
    id SERIAL PRIMARY KEY,
    coin_id VARCHAR(42) NOT NULL REFERENCES coin(id) UNIQUE,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);



CREATE INDEX king_coin_id_index ON king (coin_id);
CREATE INDEX king_created_at_index ON king (created_at);


