-- Add migration script here
ALTER TABLE group_app_role_override DROP CONSTRAINT group_app_role_override_pkey;
ALTER TABLE group_app_role_override ADD PRIMARY KEY (group_id, client_id, role);

CREATE INDEX idx_grp_role_override_by_client ON group_app_role_override(client_id);
CREATE INDEX idx_grp_role_override_by_user_client ON group_app_role_override(group_id, client_id);