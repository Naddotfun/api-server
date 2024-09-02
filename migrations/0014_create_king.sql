CREATE TABLE IF NOT EXISTS king (
    id SERIAL PRIMARY KEY,
    token_id VARCHAR(42) NOT NULL REFERENCES token(id) UNIQUE,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP
);



CREATE INDEX king_token_id_index ON king (token_id);
CREATE INDEX king_created_at_index ON king (created_at);


