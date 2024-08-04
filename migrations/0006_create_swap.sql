CREATE TABLE IF NOT EXISTS swap (
    id SERIAL PRIMARY KEY,
    sender VARCHAR(42) NOT NULL REFERENCES account(id),
    coin_id VARCHAR(42) NOT NULL REFERENCES coin(id),
    is_buy BOOLEAN NOT NULL,
    nad_amount NUMERIC NOT NULL,
    token_amount NUMERIC NOT NULL,
    created_at BIGINT NOT NULL,
    transaction_hash VARCHAR UNIQUE NOT NULL
);


-- 이건 id 가 생성되는 거에 따라서 자동 정렬되어잇음. 
-- --coin_swap_table
-- user_swap_table 
CREATE INDEX swap_coin_id_index ON swap (coin_id);
CREATE INDEX swap_created_at_index ON swap (created_at);
CREATE INDEX swap_sender_index ON swap (sender);
