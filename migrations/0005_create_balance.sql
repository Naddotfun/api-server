CREATE TABLE IF NOT EXISTS balance (
    id SERIAL PRIMARY KEY,
    coin_id VARCHAR(42) NOT NULL REFERENCES coin(id),
    account VARCHAR(42) NOT NULL REFERENCES account(id),
    amount NUMERIC NOT NULL,
    UNIQUE (account, coin_id)
);


CREATE INDEX balance_coin_id_index ON balance (coin_id);
CREATE INDEX balance_account_index ON balance (account);
CREATE INDEX balance_amount_index ON balance (amount);

