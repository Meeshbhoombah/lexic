CREATE TABLE account_encryption_keys (
    account_id UUID PRIMARY KEY REFERENCES accounts(id) ON DELETE CASCADE,
    public_key BYTEA NOT NULL CHECK (octet_length(public_key) = 65),
    key_version SMALLINT NOT NULL DEFAULT 1 CHECK (key_version > 0),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
