ALTER TABLE messages
    DROP CONSTRAINT messages_client_id_unique;

ALTER TABLE messages
    ADD CONSTRAINT messages_client_id_unique
    UNIQUE (
        conversation_id,
        sender_account_id,
        client_message_id
    );
