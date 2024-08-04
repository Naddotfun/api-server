-- Create follower table
CREATE TABLE IF NOT EXISTS follow (
    id SERIAL PRIMARY KEY,
    follower_id VARCHAR(42) NOT NULL REFERENCES account(id),
    following_id VARCHAR(42) NOT NULL REFERENCES account(id),
    UNIQUE(follower_id, following_id)
    -- FOREIGN KEY (follower_id) REFERENCES account(id),
    -- FOREIGN KEY (followed_id) REFERENCES account(id)
);
-- 인덱스 추가
CREATE INDEX idx_follower_id ON follow (follower_id);
CREATE INDEX idx_following_id ON follow (following_id);