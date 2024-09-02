CREATE TABLE IF NOT EXISTS balance (
    id SERIAL PRIMARY KEY,
    token_id VARCHAR(42) NOT NULL REFERENCES token(id),
    account_id VARCHAR(42) NOT NULL REFERENCES account(id),
    amount NUMERIC NOT NULL,
    UNIQUE (account_id, token_id)
);


CREATE INDEX balance_token_id_index ON balance (token_id);
CREATE INDEX balance_account_index ON balance (account_id);
CREATE INDEX balance_amount_index ON balance (amount);

