-- Initial schema for the Rust + React boilerplate.
-- Mirrors the Prisma schema of the Next.js sibling, with two hardening
-- changes: session tokens are stored as SHA-256 hashes and backup codes are
-- stored as JSONB arrays of SHA-256 hashes.

CREATE TYPE user_status AS ENUM ('pending', 'active', 'suspended');

CREATE TABLE roles (
    id text PRIMARY KEY,
    name text NOT NULL UNIQUE,
    label text NOT NULL,
    description text,
    is_system boolean NOT NULL DEFAULT false,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE permissions (
    id text PRIMARY KEY,
    key text NOT NULL UNIQUE,
    label text NOT NULL,
    description text,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE role_permissions (
    role_id text NOT NULL REFERENCES roles (id) ON DELETE CASCADE,
    permission_id text NOT NULL REFERENCES permissions (id) ON DELETE CASCADE,
    PRIMARY KEY (role_id, permission_id)
);

CREATE TABLE users (
    id text PRIMARY KEY,
    name text NOT NULL,
    email text NOT NULL,
    email_verified boolean NOT NULL DEFAULT false,
    image text,
    role_id text REFERENCES roles (id) ON DELETE SET NULL,
    status user_status NOT NULL DEFAULT 'active',
    phone_number text,
    job_title text,
    department text,
    two_factor_enabled boolean NOT NULL DEFAULT false,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE UNIQUE INDEX users_email_lower_idx ON users (lower(email));
CREATE INDEX users_role_id_idx ON users (role_id);
CREATE INDEX users_status_idx ON users (status);

CREATE TABLE sessions (
    id text PRIMARY KEY,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    user_id text NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    expires_at timestamptz NOT NULL,
    token_hash text NOT NULL UNIQUE,
    ip_address text,
    user_agent text
);

CREATE INDEX sessions_user_id_idx ON sessions (user_id);

CREATE TABLE accounts (
    id text PRIMARY KEY,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    provider_id text NOT NULL,
    account_id text NOT NULL,
    user_id text NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    access_token text,
    refresh_token text,
    id_token text,
    access_token_expires_at timestamptz,
    refresh_token_expires_at timestamptz,
    scope text,
    password text,
    UNIQUE (provider_id, account_id)
);

CREATE INDEX accounts_user_id_idx ON accounts (user_id);

CREATE TABLE verifications (
    id text PRIMARY KEY,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    value text NOT NULL,
    expires_at timestamptz NOT NULL,
    identifier text NOT NULL
);

CREATE INDEX verifications_identifier_idx ON verifications (identifier);
CREATE INDEX verifications_value_idx ON verifications (value);
CREATE INDEX verifications_expires_at_idx ON verifications (expires_at);

CREATE TABLE two_factors (
    id text PRIMARY KEY,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    secret text NOT NULL,
    backup_codes jsonb NOT NULL DEFAULT '[]'::jsonb,
    user_id text NOT NULL UNIQUE REFERENCES users (id) ON DELETE CASCADE,
    verified boolean NOT NULL DEFAULT false,
    failed_verification_count integer NOT NULL DEFAULT 0,
    locked_until timestamptz
);

CREATE TABLE settings (
    id text PRIMARY KEY,
    key text NOT NULL UNIQUE,
    value text,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE OR REPLACE FUNCTION set_updated_at()
RETURNS trigger
LANGUAGE plpgsql
AS $$
BEGIN
    NEW.updated_at = now();
    RETURN NEW;
END;
$$;

CREATE TRIGGER users_set_updated_at
BEFORE UPDATE ON users
FOR EACH ROW EXECUTE FUNCTION set_updated_at();

CREATE TRIGGER roles_set_updated_at
BEFORE UPDATE ON roles
FOR EACH ROW EXECUTE FUNCTION set_updated_at();

CREATE TRIGGER permissions_set_updated_at
BEFORE UPDATE ON permissions
FOR EACH ROW EXECUTE FUNCTION set_updated_at();

CREATE TRIGGER sessions_set_updated_at
BEFORE UPDATE ON sessions
FOR EACH ROW EXECUTE FUNCTION set_updated_at();

CREATE TRIGGER accounts_set_updated_at
BEFORE UPDATE ON accounts
FOR EACH ROW EXECUTE FUNCTION set_updated_at();

CREATE TRIGGER verifications_set_updated_at
BEFORE UPDATE ON verifications
FOR EACH ROW EXECUTE FUNCTION set_updated_at();

CREATE TRIGGER two_factors_set_updated_at
BEFORE UPDATE ON two_factors
FOR EACH ROW EXECUTE FUNCTION set_updated_at();

CREATE TRIGGER settings_set_updated_at
BEFORE UPDATE ON settings
FOR EACH ROW EXECUTE FUNCTION set_updated_at();
