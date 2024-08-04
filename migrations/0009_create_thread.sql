CREATE TABLE IF NOT EXISTS thread(
    id SERIAL PRIMARY KEY,
    coin_id VARCHAR(42) NOT NULL REFERENCES coin(id),
    author_id VARCHAR(42) NOT NULL REFERENCES account(id),
    content TEXT NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    root_id INTEGER REFERENCES thread(id),
    likes_count INTEGER NOT NULL DEFAULT 0,
    reply_count INTEGER NOT NULL DEFAULT 0,
    image_uri VARCHAR,
    CONSTRAINT valid_parent CHECK (id != root_id)
);

CREATE INDEX idx_thread_coin_id ON thread(coin_id);
CREATE INDEX idx_thread_author_id ON thread(author_id);
CREATE INDEX idx_thread_root_id ON thread(root_id);
CREATE INDEX idx_thread_created_at ON thread(created_at);