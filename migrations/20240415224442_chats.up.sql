-- Add up migration script here
CREATE TABLE chats (
    id serial PRIMARY KEY,
    sender_id uuid REFERENCES users(id) NOT NULL,
    recipient_id uuid REFERENCES users(id) NOT NULL CHECK (recipient_id <> sender_id),
    sender_public_key bytea NOT NULL CHECK (length(sender_public_key) = 32),
    recipient_public_key bytea CHECK (length(recipient_public_key) = 32),
    created_at timestamp DEFAULT now() NOT NULL
);

CREATE UNIQUE INDEX chats_id_pair_idx ON chats(
    least(sender_id, recipient_id), greatest(sender_id, recipient_id)
);

CREATE TABLE messages (
    id serial PRIMARY KEY,
    sender_id uuid REFERENCES users(id) NOT NULL,
    recipient_id uuid REFERENCES users(id) NOT NULL CHECK (recipient_id <> sender_id),
    content bytea NOT NULL,
    created_at timestamp DEFAULT now() NOT NULL
);

CREATE INDEX messages_keyset_pag_idx ON messages
USING btree (sender_id, recipient_id, created_at DESC, id DESC);

CREATE EXTENSION IF NOT EXISTS pg_trgm;

CREATE INDEX users_username_idx ON users
USING GIST (username gist_trgm_ops(siglen=64));