CREATE TABLE permission_group_membership (
  group_id INTEGER NOT NULL,
  user_id INTEGER NOT NULL,
  PRIMARY KEY (group_id, user_id)
);

CREATE INDEX idx_users_in_permission_group ON permission_group_membership(group_id);
CREATE INDEX idx_permission_groups_for_user ON permission_group_membership(user_id);