CREATE TABLE conversations (
    id UUID PRIMARY KEY,
    kind TEXT NOT NULL
        CHECK (kind IN ('direct', 'group')),
    created_by UUID NOT NULL
        REFERENCES accounts(id) ON DELETE RESTRICT,

    -- Direct conversations use a canonical, sorted account pair.
    -- Group conversations leave both columns NULL.
    direct_account_low_id UUID
        REFERENCES accounts(id) ON DELETE RESTRICT,
    direct_account_high_id UUID
        REFERENCES accounts(id) ON DELETE RESTRICT,

    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT conversations_direct_pair_check CHECK (
        (
            kind = 'direct'
            AND direct_account_low_id IS NOT NULL
            AND direct_account_high_id IS NOT NULL
            AND direct_account_low_id < direct_account_high_id
        )
        OR
        (
            kind = 'group'
            AND direct_account_low_id IS NULL
            AND direct_account_high_id IS NULL
        )
    ),

    CONSTRAINT conversations_direct_pair_unique UNIQUE (
        direct_account_low_id,
        direct_account_high_id
    )
);

CREATE TABLE conversation_participants (
    conversation_id UUID NOT NULL
        REFERENCES conversations(id) ON DELETE CASCADE,
    account_id UUID NOT NULL
        REFERENCES accounts(id) ON DELETE CASCADE,

    role TEXT NOT NULL DEFAULT 'member'
        CHECK (role IN ('owner', 'member')),

    joined_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    last_read_at TIMESTAMPTZ,

    PRIMARY KEY (conversation_id, account_id)
);

CREATE INDEX conversation_participants_account_idx
    ON conversation_participants(account_id, joined_at DESC);

CREATE TABLE messages (
    id UUID PRIMARY KEY,

    conversation_id UUID NOT NULL,
    sender_account_id UUID NOT NULL,
    client_message_id UUID NOT NULL,

    -- Version the client/server message-envelope protocol.
    protocol_version SMALLINT NOT NULL DEFAULT 1
        CHECK (protocol_version > 0),

    -- Opaque cryptographic material; the server must not decrypt it.
    nonce BYTEA NOT NULL,
    ciphertext BYTEA NOT NULL
        CHECK (octet_length(ciphertext) > 0),

    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT messages_sender_must_be_participant
        FOREIGN KEY (conversation_id, sender_account_id)
        REFERENCES conversation_participants(conversation_id, account_id)
        ON DELETE CASCADE,

    -- Makes client retries idempotent for a given sender.
    CONSTRAINT messages_client_id_unique
        UNIQUE (sender_account_id, client_message_id)
);

CREATE INDEX messages_conversation_history_idx
    ON messages(conversation_id, created_at DESC, id DESC);
