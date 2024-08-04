CREATE TABLE IF NOT EXISTS coin_reply_count (
    coin_id VARCHAR PRIMARY KEY REFERENCES coin(id),
    reply_count INTEGER NOT NULL DEFAULT 0
);

CREATE INDEX coin_replies_count_coin_id_index ON coin_reply_count (coin_id);
CREATE INDEX coin_replies_count_reply_count_index ON  coin_reply_count(reply_count);