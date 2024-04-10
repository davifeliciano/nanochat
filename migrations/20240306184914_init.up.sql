-- Add up migration script here
CREATE TABLE users (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    username varchar(32) UNIQUE NOT NULL CHECK (username ~ '^[\w\-_]{2,32}$'),
    password varchar(256) NOT NULL,
    pbkdf2_salt varchar(256) NOT NULL,
    created_at timestamp DEFAULT now() NOT NULL
);

CREATE TABLE sessions (
    id serial PRIMARY KEY,
    token varchar(512) NOT NULL,
    user_id uuid REFERENCES users(id) NOT NULL,
    created_at timestamp DEFAULT now() NOT NULL
);
