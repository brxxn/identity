-- we will probably need to add more to this later, but just keeping
-- the essentials for now.
CREATE TABLE user_sessions (
  session_id BIGINT NOT NULL PRIMARY KEY,
  user_id INTEGER NOT NULL,
  refresh_salt TEXT NOT NULL,
  refresh_hash TEXT NOT NULL
);

CREATE INDEX idx_sessions_by_user ON user_sessions(user_id);