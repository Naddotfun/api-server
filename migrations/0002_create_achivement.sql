CREATE TABLE IF NOT EXISTS achievement (
    id SERIAL PRIMARY KEY,
    name VARCHAR(15) UNIQUE NOT NULL,
    image_uri VARCHAR NOT NULL
);

-- account와 achievement를 연결하는 중간 테이블
CREATE TABLE IF NOT EXISTS account_achievement (
    account_id VARCHAR(42) REFERENCES account(id),
    achievement_id INT REFERENCES achievement(id),
    PRIMARY KEY (account_id, achievement_id)
);

CREATE INDEX achivement_name_index ON achievement (name);
CREATE INDEX account_achievement_account_id_index ON account_achievement (account_id);
CREATE INDEX account_achievement_achievement_id_index ON account_achievement (achievement_id);




INSERT INTO achievement (name, image_uri) VALUES
('chad_creator', 'https://pub-56950a3ba13e4c43ba0b2e803fd9b2f1.r2.dev/chad_creator.jpg'),
('rugger', 'https://pub-56950a3ba13e4c43ba0b2e803fd9b2f1.r2.dev/rugger.jpg'),
('left_curve','https://pub-56950a3ba13e4c43ba0b2e803fd9b2f1.r2.dev/left_curve.jpg')
ON CONFLICT (name) DO NOTHING;

