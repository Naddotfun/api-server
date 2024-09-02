CREATE TABLE IF NOT EXISTS thread_likes (
    id SERIAL PRIMARY KEY,
    thread_id INTEGER NOT NULL REFERENCES thread(id),
    user_id VARCHAR(42) NOT NULL REFERENCES account(id),
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE (thread_id, user_id)
);


CREATE INDEX idx_thread_likes_thread_id ON thread_likes(thread_id);
CREATE INDEX idx_thread_likes_user_id ON thread_likes(user_id);
CREATE INDEX idx_thread_likes_created_at ON thread_likes(created_at);