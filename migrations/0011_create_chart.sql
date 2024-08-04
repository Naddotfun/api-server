CREATE TABLE IF NOT EXISTS chart_1m (
    id SERIAL PRIMARY KEY,
    coin_id VARCHAR(42) NOT NULL REFERENCES coin(id),
    open_price NUMERIC(15,10) NOT NULL,
    close_price NUMERIC(15,10) NOT NULL,
    high_price NUMERIC(15,10) NOT NULL,
    low_price NUMERIC(15,10) NOT NULL,
    created_at BIGINT NOT NULL, 
    UNIQUE (coin_id, created_at)
);

CREATE TABLE IF NOT EXISTS chart_5m (
    id SERIAL PRIMARY KEY,
    coin_id VARCHAR(42) NOT NULL REFERENCES coin(id),
    open_price NUMERIC(15,10) NOT NULL,
    close_price NUMERIC(15,10) NOT NULL,
    high_price NUMERIC(15,10) NOT NULL,
    low_price NUMERIC(15,10) NOT NULL,
    created_at BIGINT NOT NULL, 
    UNIQUE (coin_id, created_at)
);

CREATE TABLE IF NOT EXISTS chart_15m (
    id SERIAL PRIMARY KEY,
    coin_id VARCHAR(42) NOT NULL REFERENCES coin(id),
    open_price NUMERIC(15,10) NOT NULL,
    close_price NUMERIC(15,10) NOT NULL,
    high_price NUMERIC(15,10) NOT NULL,
    low_price NUMERIC(15,10) NOT NULL,
    created_at BIGINT NOT NULL, 
    UNIQUE (coin_id, created_at)
);

CREATE TABLE IF NOT EXISTS chart_30m (
    id SERIAL PRIMARY KEY,
    coin_id VARCHAR(42) NOT NULL REFERENCES coin(id),
    open_price NUMERIC(15,10) NOT NULL,
    close_price NUMERIC(15,10) NOT NULL,
    high_price NUMERIC(15,10) NOT NULL,
    low_price NUMERIC(15,10) NOT NULL,
    created_at BIGINT NOT NULL, 
    UNIQUE (coin_id, created_at)
);

CREATE TABLE IF NOT EXISTS chart_1h (
    id SERIAL PRIMARY KEY,
    coin_id VARCHAR(42) NOT NULL REFERENCES coin(id),
    open_price NUMERIC(15,10) NOT NULL,
    close_price NUMERIC(15,10) NOT NULL,
    high_price NUMERIC(15,10) NOT NULL,
    low_price NUMERIC(15,10) NOT NULL,
    created_at BIGINT NOT NULL, 
    UNIQUE (coin_id, created_at)
);

CREATE TABLE IF NOT EXISTS chart_4h (
    id SERIAL PRIMARY KEY,
    coin_id VARCHAR(42) NOT NULL REFERENCES coin(id),
    open_price NUMERIC(15,10) NOT NULL,
    close_price NUMERIC(15,10) NOT NULL,
    high_price NUMERIC(15,10) NOT NULL,
    low_price NUMERIC(15,10) NOT NULL,
    created_at BIGINT NOT NULL,
    UNIQUE (coin_id, created_at) 
);

CREATE TABLE IF NOT EXISTS chart_1d (
    id SERIAL PRIMARY KEY,
    coin_id VARCHAR(42) NOT NULL REFERENCES coin(id),
    open_price NUMERIC(15,10) NOT NULL,
    close_price NUMERIC(15,10) NOT NULL,
    high_price NUMERIC(15,10) NOT NULL,
    low_price NUMERIC(15,10) NOT NULL,
    created_at BIGINT NOT NULL,
    UNIQUE (coin_id, created_at)  
);


CREATE INDEX idx_chart_1m_coin_id ON chart_1m(coin_id);
CREATE INDEX idx_chart_1m_coin_id_created_at ON chart_1m(coin_id, created_at);

CREATE INDEX idx_chart_5m_coin_id ON chart_5m(coin_id);
CREATE INDEX idx_chart_5m_coin_id_created_at ON chart_5m(coin_id, created_at);

CREATE INDEX idx_chart_15m_coin_id ON chart_15m(coin_id);
CREATE INDEX idx_chart_15m_coin_id_created_at ON chart_15m(coin_id, created_at);

CREATE INDEX idx_chart_30m_coin_id ON chart_30m(coin_id);
CREATE INDEX idx_chart_30m_coin_id_created_at ON chart_30m(coin_id, created_at);

CREATE INDEX idx_chart_1h_coin_id ON chart_1h(coin_id);
CREATE INDEX idx_chart_1h_coin_id_created_at ON chart_1h(coin_id, created_at);

CREATE INDEX idx_chart_4h_coin_id ON chart_4h(coin_id);
CREATE INDEX idx_chart_4h_coin_id_created_at ON chart_4h(coin_id, created_at);

CREATE INDEX idx_chart_1d_coin_id ON chart_1d(coin_id);
CREATE INDEX idx_chart_1d_coin_id_created_at ON chart_1d(coin_id, created_at);