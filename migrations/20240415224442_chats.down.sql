-- Add down migration script here
DROP INDEX users_username_idx;
DROP EXTENSION pg_trgm;
DROP INDEX messages_keyset_pag_idx;
DROP TABLE messages;
DROP TABLE chats;