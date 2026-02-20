use std::error::Error;

use serde::{Deserialize, Serialize};
use sqlx::PgPool;

#[derive(Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct UserPermissionOverride {
  pub user_id: i32,
  pub client_id: String,
  pub granted: bool,
}

#[derive(Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct GroupPermissionOverride {
  pub group_id: i32,
  pub client_id: String,
  pub granted: bool,
  pub override_priority: i32,
}

impl UserPermissionOverride {
  pub async fn fetch_user_permissions_for_client(
    pool: &PgPool,
    user_id: i32,
    client_id: String,
  ) -> Result<Option<UserPermissionOverride>, Box<dyn Error>> {
    let permission_override = sqlx::query_as!(
      UserPermissionOverride,
      r#"
        SELECT user_id, client_id, granted FROM user_app_permission_override WHERE user_id = $1 AND client_id = $2
      "#,
      user_id,
      client_id
    ).fetch_optional(pool).await?;
    Ok(permission_override)
  }

  pub async fn upsert_permission_override(&self, pool: &PgPool) -> Result<(), Box<dyn Error>> {
    sqlx::query!(
      r#"
        INSERT INTO user_app_permission_override(user_id, client_id, granted) VALUES ($1, $2, $3)
        ON CONFLICT (user_id, client_id) DO UPDATE SET granted = EXCLUDED.granted
      "#,
      self.user_id,
      self.client_id,
      self.granted
    )
    .execute(pool)
    .await?;
    Ok(())
  }

  pub async fn remove_permission_override(
    pool: &PgPool,
    user_id: i32,
    client_id: String,
  ) -> Result<(), Box<dyn Error>> {
    sqlx::query!(
      r#"
        DELETE FROM user_app_permission_override WHERE user_id = $1 AND client_id = $2
      "#,
      user_id,
      client_id
    )
    .execute(pool)
    .await?;
    Ok(())
  }

  pub async fn get_overrides_for_client(
    pool: &PgPool,
    client_id: String,
  ) -> Result<Vec<Self>, Box<dyn Error>> {
    let permission_overrides = sqlx::query_as!(
      UserPermissionOverride,
      r#"
        SELECT user_id, client_id, granted FROM user_app_permission_override WHERE client_id = $1
      "#,
      client_id
    )
    .fetch_all(pool)
    .await?;
    Ok(permission_overrides)
  }
}

impl GroupPermissionOverride {
  pub async fn fetch_group_permissions_for_client(
    pool: &PgPool,
    client_id: String,
  ) -> Result<Vec<GroupPermissionOverride>, Box<dyn Error>> {
    let permission_overrides = sqlx::query_as!(
      GroupPermissionOverride,
      r#"
        SELECT group_id, client_id, granted, override_priority FROM group_app_permission_override WHERE client_id = $1
        ORDER BY override_priority ASC
      "#,
      client_id
    ).fetch_all(pool).await?;
    Ok(permission_overrides)
  }

  pub async fn upsert_permission_override(&self, pool: &PgPool) -> Result<(), Box<dyn Error>> {
    sqlx::query!(
      r#"
        INSERT INTO group_app_permission_override(group_id, client_id, granted, override_priority) VALUES ($1, $2, $3, $4)
        ON CONFLICT (group_id, client_id) DO UPDATE SET granted = EXCLUDED.granted, override_priority = EXCLUDED.override_priority
      "#,
      self.group_id,
      self.client_id,
      self.granted,
      self.override_priority
    ).execute(pool).await?;
    Ok(())
  }

  pub async fn remove_permission_override(
    pool: &PgPool,
    group_id: i32,
    client_id: String,
  ) -> Result<(), Box<dyn Error>> {
    sqlx::query!(
      r#"
        DELETE FROM group_app_permission_override WHERE group_id = $1 AND client_id = $2
      "#,
      group_id,
      client_id
    )
    .execute(pool)
    .await?;
    Ok(())
  }

  pub async fn set_group_overrides_for_client(
    pool: &PgPool,
    overrides: &[GroupPermissionOverride],
    client_id: String,
  ) -> Result<(), Box<dyn Error>> {
    let mut transaction = pool.begin().await?;
    sqlx::query!(
      r#"
        DELETE FROM group_app_permission_override WHERE client_id = $1
      "#,
      client_id
    )
    .execute(&mut *transaction)
    .await?;
    for override_entry in overrides {
      sqlx::query!(
        r#"
          INSERT INTO group_app_permission_override(group_id, client_id, granted, override_priority) VALUES ($1, $2, $3, $4)
          ON CONFLICT (group_id, client_id) DO UPDATE SET granted = EXCLUDED.granted, override_priority = EXCLUDED.override_priority
        "#,
        override_entry.group_id,
        override_entry.client_id,
        override_entry.granted,
        override_entry.override_priority
      ).execute(&mut *transaction).await?;
    }
    transaction.commit().await?;
    Ok(())
  }
}
