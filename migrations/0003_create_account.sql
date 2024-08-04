CREATE TABLE IF NOT EXISTS account (
    id VARCHAR(42) PRIMARY KEY,
    nickname VARCHAR(42) NOT NULL UNIQUE,
    image_uri VARCHAR NOT NULL,
    follower_count INT NOT NULL DEFAULT 0,
    following_count INT NOT NULL DEFAULT 0,
    like_count INT NOT NULL DEFAULT 0
);



CREATE INDEX account_id_index ON account (id);
CREATE INDEX account_nickname_index ON account (nickname);
CREATE INDEX account_follower_count_index ON account (follower_count);
CREATE INDEX account_like_count_index ON account (like_count);