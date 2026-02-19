CREATE TABLE users (
  id SERIAL PRIMARY KEY,
  username TEXT NOT NULL UNIQUE,
  email TEXT NOT NULL UNIQUE,
  name TEXT NOT NULL,
  is_suspended BOOLEAN NOT NULL DEFAULT FALSE,
  credential_uuid UUID NOT NULL UNIQUE
);

CREATE TABLE user_webauthn_credentials (
  credential_uuid UUID NOT NULL,
  name TEXT NOT NULL,
  credential_id TEXT NOT NULL,
  serialized_passkey TEXT NOT NULL,
  CONSTRAINT fk_credential_uuid
    FOREIGN KEY (credential_uuid)
    REFERENCES users(credential_uuid)
);

CREATE INDEX idx_user_webauthn_credentials ON user_webauthn_credentials(credential_uuid);

CREATE TABLE clients (
  client_id TEXT NOT NULL PRIMARY KEY,
  client_secret TEXT NOT NULL,
  app_name TEXT NOT NULL,
  redirect_uris TEXT[] NOT NULL,
  grant_types TEXT[] NOT NULL,
  role_claim_override TEXT NOT NULL,
  registered_roles TEXT[] NOT NULL,
  is_managed BOOLEAN NOT NULL DEFAULT FALSE,
  is_disabled BOOLEAN NOT NULL DEFAULT FALSE,
  default_allowed BOOLEAN NOT NULL DEFAULT FALSE
);

CREATE TABLE permission_groups (
  id SERIAL PRIMARY KEY,
  slug TEXT NOT NULL UNIQUE,
  name TEXT NOT NULL,
  description TEXT NOT NULL,
  is_managed BOOLEAN NOT NULL DEFAULT FALSE,
  is_admin_group BOOLEAN NOT NULL DEFAULT FALSE
);

CREATE TABLE group_app_permission_override (
  group_id INTEGER NOT NULL REFERENCES permission_groups(id),
  client_id TEXT NOT NULL REFERENCES clients(client_id),
  granted BOOLEAN NOT NULL DEFAULT TRUE,
  PRIMARY KEY (group_id, client_id)
);

CREATE TABLE user_app_permission_override (
  user_id INTEGER NOT NULL REFERENCES users(id),
  client_id TEXT NOT NULL REFERENCES clients(client_id),
  granted BOOLEAN NOT NULL DEFAULT TRUE,
  PRIMARY KEY (user_id, client_id)
);

CREATE TABLE group_app_role_override (
  group_id INTEGER NOT NULL REFERENCES permission_groups(id),
  client_id TEXT NOT NULL REFERENCES clients(client_id),
  role TEXT NOT NULL,
  granted BOOLEAN NOT NULL DEFAULT TRUE,
  PRIMARY KEY (group_id, client_id)
);

CREATE TABLE user_app_role_override (
  user_id INTEGER NOT NULL REFERENCES users(id),
  client_id TEXT NOT NULL REFERENCES clients(client_id),
  role TEXT NOT NULL,
  granted BOOLEAN NOT NULL DEFAULT TRUE,
  PRIMARY KEY (user_id, client_id)
);

