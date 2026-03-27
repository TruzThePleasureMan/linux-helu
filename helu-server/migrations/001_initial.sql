-- migrations/001_initial.sql

CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

CREATE TABLE users (
    id          UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    username    TEXT NOT NULL UNIQUE,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    enabled     BOOLEAN NOT NULL DEFAULT TRUE
);

CREATE TABLE api_keys (
    id          UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name        TEXT NOT NULL,              -- e.g. "vpn-gateway-prod"
    key_hash    TEXT NOT NULL,              -- Argon2 hash of the raw key
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    last_used   TIMESTAMPTZ,
    enabled     BOOLEAN NOT NULL DEFAULT TRUE
);

CREATE TABLE auth_log (
    id          UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    username    TEXT NOT NULL,
    method      TEXT NOT NULL,              -- face, fingerprint, fido2, pin
    success     BOOLEAN NOT NULL,
    reason      TEXT,
    client_ip   INET,
    api_key_id  UUID REFERENCES api_keys(id),
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE challenges (
    id          UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    username    TEXT NOT NULL,
    nonce       TEXT NOT NULL,
    issued_at   TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_at  TIMESTAMPTZ NOT NULL,
    used        BOOLEAN NOT NULL DEFAULT FALSE
);
