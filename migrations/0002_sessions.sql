CREATE TABLE sessions (
    id UUID PRIMARY KEY,
    account_id UUID NOT NULL
        REFERENCES accounts(id) ON DELETE CASCADE,
    token_hash BYTEA NOT NULL UNIQUE,
    expires_at TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX sessions_account_id_idx
    ON sessions(account_id);

CREATE INDEX sessions_expires_at_idx
    ON sessions(expires_at);
