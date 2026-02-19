CREATE TABLE user_app_authorizations (
  user_id INTEGER NOT NULL,
  client_id TEXT NOT NULL,
  sub TEXT NOT NULL UNIQUE,
  last_used BIGINT NOT NULL,
  PRIMARY KEY (user_id, client_id)
);

CREATE INDEX idx_authorized_apps_by_user ON user_app_authorizations(user_id);