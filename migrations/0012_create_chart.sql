CREATE TABLE IF NOT EXISTS chart_1m (
    id SERIAL PRIMARY KEY,
    token_id VARCHAR(42) NOT NULL REFERENCES token(id),
    open_price NUMERIC(15,10) NOT NULL,
    close_price NUMERIC(15,10) NOT NULL,
    high_price NUMERIC(15,10) NOT NULL,
    low_price NUMERIC(15,10) NOT NULL,
    time_stamp BIGINT NOT NULL, 
    UNIQUE (token_id, time_stamp)
);

CREATE TABLE IF NOT EXISTS chart_5m (
    id SERIAL PRIMARY KEY,
    token_id VARCHAR(42) NOT NULL REFERENCES token(id),
    open_price NUMERIC(15,10) NOT NULL,
    close_price NUMERIC(15,10) NOT NULL,
    high_price NUMERIC(15,10) NOT NULL,
    low_price NUMERIC(15,10) NOT NULL,
    time_stamp BIGINT NOT NULL, 
    UNIQUE (token_id, time_stamp)
);

CREATE TABLE IF NOT EXISTS chart_15m (
    id SERIAL PRIMARY KEY,
    token_id VARCHAR(42) NOT NULL REFERENCES token(id),
    open_price NUMERIC(15,10) NOT NULL,
    close_price NUMERIC(15,10) NOT NULL,
    high_price NUMERIC(15,10) NOT NULL,
    low_price NUMERIC(15,10) NOT NULL,
    time_stamp BIGINT NOT NULL, 
    UNIQUE (token_id, time_stamp)
);

CREATE TABLE IF NOT EXISTS chart_30m (
    id SERIAL PRIMARY KEY,
    token_id VARCHAR(42) NOT NULL REFERENCES token(id),
    open_price NUMERIC(15,10) NOT NULL,
    close_price NUMERIC(15,10) NOT NULL,
    high_price NUMERIC(15,10) NOT NULL,
    low_price NUMERIC(15,10) NOT NULL,
    time_stamp BIGINT NOT NULL, 
    UNIQUE (token_id, time_stamp)
);

CREATE TABLE IF NOT EXISTS chart_1h (
    id SERIAL PRIMARY KEY,
    token_id VARCHAR(42) NOT NULL REFERENCES token(id),
    open_price NUMERIC(15,10) NOT NULL,
    close_price NUMERIC(15,10) NOT NULL,
    high_price NUMERIC(15,10) NOT NULL,
    low_price NUMERIC(15,10) NOT NULL,
    time_stamp BIGINT NOT NULL, 
    UNIQUE (token_id, time_stamp)
);

CREATE TABLE IF NOT EXISTS chart_4h (
    id SERIAL PRIMARY KEY,
    token_id VARCHAR(42) NOT NULL REFERENCES token(id),
    open_price NUMERIC(15,10) NOT NULL,
    close_price NUMERIC(15,10) NOT NULL,
    high_price NUMERIC(15,10) NOT NULL,
    low_price NUMERIC(15,10) NOT NULL,
    time_stamp BIGINT NOT NULL,
    UNIQUE (token_id, time_stamp) 
);

CREATE TABLE IF NOT EXISTS chart_1d (
    id SERIAL PRIMARY KEY,
    token_id VARCHAR(42) NOT NULL REFERENCES token(id),
    open_price NUMERIC(15,10) NOT NULL,
    close_price NUMERIC(15,10) NOT NULL,
    high_price NUMERIC(15,10) NOT NULL,
    low_price NUMERIC(15,10) NOT NULL,
    time_stamp BIGINT NOT NULL,
    UNIQUE (token_id, time_stamp)  
);


CREATE INDEX idx_chart_1m_token_id ON chart_1m(token_id);
CREATE INDEX idx_chart_1m_token_id_time_stamp ON chart_1m(token_id, time_stamp);

CREATE INDEX idx_chart_5m_token_id ON chart_5m(token_id);
CREATE INDEX idx_chart_5m_token_id_time_stamp ON chart_5m(token_id, time_stamp);

CREATE INDEX idx_chart_15m_token_id ON chart_15m(token_id);
CREATE INDEX idx_chart_15m_token_id_time_stamp ON chart_15m(token_id, time_stamp);

CREATE INDEX idx_chart_30m_token_id ON chart_30m(token_id);
CREATE INDEX idx_chart_30m_token_id_time_stamp ON chart_30m(token_id, time_stamp);

CREATE INDEX idx_chart_1h_token_id ON chart_1h(token_id);
CREATE INDEX idx_chart_1h_token_id_time_stamp ON chart_1h(token_id, time_stamp);

CREATE INDEX idx_chart_4h_token_id ON chart_4h(token_id);
CREATE INDEX idx_chart_4h_token_id_time_stamp ON chart_4h(token_id, time_stamp);

CREATE INDEX idx_chart_1d_token_id ON chart_1d(token_id);
CREATE INDEX idx_chart_1d_token_id_time_stamp ON chart_1d(token_id, time_stamp);