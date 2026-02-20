use std::error::Error;

use serde::{Deserialize, Serialize};
use sqlx::PgPool;

#[derive(Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct UserAppRoleOverride {
  pub user_id: i32,
  pub client_id: String,
  pub role: String,
  pub granted: bool,
}

#[derive(Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct GroupAppRoleOverride {
  pub group_id: i32,
  pub client_id: String,
  pub role: String,
  pub granted: bool,
  pub override_priority: i32,
}

impl UserAppRoleOverride {
  pub async fn fetch_user_role_overrides_for_client(
    pool: &PgPool,
    user_id: i32,
    client_id: String,
  ) -> Result<Vec<UserAppRoleOverride>, Box<dyn Error>> {
    let role_overrides = sqlx::query_as!(
      UserAppRoleOverride,
      r#"
        SELECT user_id, client_id, role, granted FROM user_app_role_override WHERE user_id = $1 AND client_id = $2
      "#,
      user_id,
      client_id
    ).fetch_all(pool).await?;
    Ok(role_overrides)
  }

  pub async fn upsert_user_role_override(&self, pool: &PgPool) -> Result<(), Box<dyn Error>> {
    sqlx::query!(
      r#"
        INSERT INTO user_app_role_override(user_id, client_id, role, granted) VALUES ($1, $2, $3, $4)
        ON CONFLICT (user_id, client_id, role) DO UPDATE SET granted = EXCLUDED.granted
      "#,
      self.user_id,
      self.client_id,
      self.role,
      self.granted
    ).execute(pool).await?;
    Ok(())
  }

  pub async fn get_overrides_for_client(
    pool: &PgPool,
    client_id: String,
  ) -> Result<Vec<UserAppRoleOverride>, Box<dyn Error>> {
    let role_overrides = sqlx::query_as!(
      UserAppRoleOverride,
      r#"
        SELECT user_id, client_id, role, granted FROM user_app_role_override WHERE client_id=$1
      "#,
      client_id
    )
    .fetch_all(pool)
    .await?;
    Ok(role_overrides)
  }

  pub async fn remove_override(&self, pool: &PgPool) -> Result<(), Box<dyn Error>> {
    sqlx::query!(
      r#"
        DELETE FROM user_app_role_override WHERE user_id = $1 AND client_id = $2 AND role = $3
      "#,
      self.user_id,
      self.client_id,
      self.role
    )
    .execute(pool)
    .await?;
    Ok(())
  }
}

impl GroupAppRoleOverride {
  pub async fn fetch_group_role_overrides_for_client(
    pool: &PgPool,
    client_id: String,
  ) -> Result<Vec<GroupAppRoleOverride>, Box<dyn Error>> {
    let role_overrides = sqlx::query_as!(
      GroupAppRoleOverride,
      r#"
        SELECT group_id, client_id, role, granted, override_priority FROM group_app_role_override WHERE client_id = $1
      "#,
      client_id
    ).fetch_all(pool).await?;
    Ok(role_overrides)
  }

  pub async fn upsert_group_role_overrides_for_client(
    pool: &PgPool,
    client_id: String,
    overrides: Vec<GroupAppRoleOverride>,
  ) -> Result<(), Box<dyn Error>> {
    let mut transaction = pool.begin().await?;
    sqlx::query!(
      r#"
        DELETE FROM group_app_role_override WHERE client_id = $1
      "#,
      client_id
    )
    .execute(&mut *transaction)
    .await?;
    for override_entry in overrides {
      sqlx::query!(
        r#"
          INSERT INTO group_app_role_override(group_id, client_id, role, granted, override_priority) VALUES ($1, $2, $3, $4, $5)
          ON CONFLICT (group_id, client_id, role) DO UPDATE SET granted = EXCLUDED.granted, override_priority = EXCLUDED.override_priority
        "#,
        override_entry.group_id,
        override_entry.client_id,
        override_entry.role,
        override_entry.granted,
        override_entry.override_priority
      ).execute(&mut *transaction).await?;
    }
    transaction.commit().await?;
    Ok(())
  }
}
