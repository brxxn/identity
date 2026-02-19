ALTER TABLE clients DROP COLUMN role_claim_override;
ALTER TABLE clients DROP COLUMN registered_roles;
ALTER TABLE clients ADD COLUMN app_desciption TEXT NOT NULL;

ALTER TABLE group_app_permission_override ADD COLUMN override_priority INTEGER NOT NULL DEFAULT 0;
ALTER TABLE group_app_role_override ADD COLUMN override_priority INTEGER NOT NULL DEFAULT 0;