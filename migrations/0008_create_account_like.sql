-- Create follower table
CREATE TABLE IF NOT EXISTS account_like (
    id SERIAL PRIMARY KEY,
    liker_id VARCHAR(42) NOT NULL REFERENCES account(id),
    liking_id VARCHAR(42) NOT NULL REFERENCES account(id),
    UNIQUE(liker_id, liking_id)

);
-- 인덱스 추가
CREATE INDEX idx_liker_id_id ON account_like (liker_id);
CREATE INDEX idx_liking_id_id ON account_like (liking_id);