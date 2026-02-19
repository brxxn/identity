-- default 0 is used to avoid breaking existing columns.
ALTER TABLE user_sessions ADD COLUMN webauthn_id INTEGER NOT NULL DEFAULT 0;

-- this is needed so we can support quickly deleting all sessions for a webauthn
-- credential id.
CREATE INDEX idx_user_sessions_by_webauthn ON user_sessions(webauthn_id);