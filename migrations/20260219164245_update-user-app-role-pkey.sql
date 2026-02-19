ALTER TABLE user_app_role_override DROP CONSTRAINT user_app_role_override_pkey;
ALTER TABLE user_app_role_override ADD PRIMARY KEY (user_id, client_id, role);

CREATE INDEX idx_role_override_by_client ON user_app_role_override(client_id);
CREATE INDEX idx_role_override_by_user_client ON user_app_role_override(user_id, client_id);