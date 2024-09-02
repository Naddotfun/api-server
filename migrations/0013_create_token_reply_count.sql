CREATE TABLE IF NOT EXISTS token_reply_count (
    token_id VARCHAR PRIMARY KEY REFERENCES token(id),
    reply_count INTEGER NOT NULL DEFAULT 0
);

CREATE INDEX token_replies_count_token_id_index ON token_reply_count (token_id);
CREATE INDEX token_replies_count_reply_count_index ON  token_reply_count(reply_count);