use std::error::Error;

use rand::distributions::{Alphanumeric, DistString};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

#[derive(Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct UserAppAuthorization {
  pub user_id: i32,
  pub client_id: String,
  pub sub: String,
  pub last_used: i64,
  pub revoked: bool,
}

impl UserAppAuthorization {
  pub async fn authorize_for_user(&mut self, pool: &PgPool) -> Result<(), Box<dyn Error>> {
    let timestamp = std::time::SystemTime::now()
      .duration_since(std::time::SystemTime::UNIX_EPOCH)
      .expect("time has somehow gone backwards...")
      .as_secs();

    // these values will be overriden if present (mostly)
    self.sub = Alphanumeric.sample_string(&mut rand::thread_rng(), 64);
    self.revoked = false;
    self.last_used = timestamp as i64;
    let authorization = sqlx::query_as!(
      UserAppAuthorization,
      r#"
        INSERT INTO user_app_authorizations(user_id, client_id, sub, last_used, revoked) VALUES ($1, $2, $3, $4, $5)
        ON CONFLICT (user_id, client_id) DO UPDATE SET last_used = EXCLUDED.last_used, revoked = EXCLUDED.revoked
        RETURNING *
      "#,
      self.user_id,
      self.client_id,
      self.sub,
      self.last_used,
      self.revoked
    )
    .fetch_one(pool)
    .await?;
    self.sub = authorization.sub;
    Ok(())
  }

  pub async fn revoke_app_authorization(pool: &PgPool, user_id: i32, client_id: String) -> Result<(), Box<dyn Error>> {
    sqlx::query!(
      r#"
        UPDATE user_app_authorizations SET revoked = TRUE WHERE user_id = $1 AND client_id = $2
      "#,
      user_id,
      client_id
    )
    .execute(pool)
    .await?;
    Ok(())
  }

  pub async fn get_authorizations_for_user(pool: &PgPool, user_id: i32) -> Result<Vec<UserAppAuthorization>, Box<dyn Error>> {
    let authorizations = sqlx::query_as!(
      UserAppAuthorization,
      r#"
        SELECT 
          user_id, client_id, sub, last_used, revoked
        FROM user_app_authorizations WHERE user_id = $1
      "#,
      user_id
    )
    .fetch_all(pool)
    .await?;
    Ok(authorizations)
  }

  pub async fn get_authorization(pool: &PgPool, user_id: i32, client_id: String) -> Result<UserAppAuthorization, Box<dyn Error>> {
    let authorizations = sqlx::query_as!(
      UserAppAuthorization,
      r#"
        SELECT 
          user_id, client_id, sub, last_used, revoked
        FROM user_app_authorizations WHERE user_id = $1 AND client_id = $2
      "#,
      user_id,
      client_id
    )
    .fetch_one(pool)
    .await?;
    Ok(authorizations)
  }
}
