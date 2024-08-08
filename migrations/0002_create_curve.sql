CREATE TABLE IF NOT EXISTS curve (
    id VARCHAR PRIMARY KEY,
    coin_id VARCHAR NOT NULL REFERENCES coin(id),
    virtual_nad NUMERIC NOT NULL,
    virtual_token NUMERIC NOT NULL,
    latest_trade_at BIGINT NOT NULL,
    price NUMERIC(15,10)  NOT NULL,
    created_at BIGINT NOT NULL
);
CREATE INDEX curve_coin_id_index ON curve (coin_id);
CREATE INDEX curve_latest_trade_at_index ON curve (latest_trade_at);
CREATE INDEX curve_created_at_index ON curve (created_at);
CREATE INDEX curve_price_index ON curve (price);