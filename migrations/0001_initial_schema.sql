CREATE TABLE accounts (
    id UUID PRIMARY KEY,
    status TEXT NOT NULL DEFAULT 'active'
        CHECK (status IN ('active', 'suspended', 'deleted')),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE account_connections (
    id UUID PRIMARY KEY,
    account_id UUID NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    provider TEXT NOT NULL CHECK (provider = 'lexe'),
    connected_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX account_connections_account_id_idx
    ON account_connections(account_id);

