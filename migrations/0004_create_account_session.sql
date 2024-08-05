CREATE TABLE IF NOT EXISTS account_session (
    id VARCHAR(32) PRIMARY KEY,
    account_id VARCHAR(42) UNIQUE NOT NULL,
    FOREIGN KEY (account_id) REFERENCES account(id) ON DELETE CASCADE
);



CREATE INDEX account_session_account_id_index ON account_session (account_id);
