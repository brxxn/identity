use std::error::Error;

use argon2::{Argon2, PasswordHasher, password_hash::SaltString};
use rand::distributions::{Alphanumeric, DistString};
use serde::{Deserialize, Serialize};
use serde_with::{DisplayFromStr, serde_as};
use sqlx::PgPool;
use tokio::task::spawn_blocking;

#[serde_as]
#[derive(Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct UserSession {
  #[serde_as(as = "DisplayFromStr")]
  pub session_id: i64,
  pub user_id: i32,
  #[serde(skip)]
  pub refresh_hash: String,
  pub webauthn_id: i32,
}

impl UserSession {
  pub async fn from_user_id(
    pool: &PgPool,
    user_id: i32,
  ) -> Result<Vec<UserSession>, Box<dyn Error>> {
    let credentials = sqlx::query_as!(
      UserSession,
      r#"
        SELECT 
          session_id, user_id, refresh_hash, webauthn_id
        FROM user_sessions WHERE user_id = $1
      "#,
      user_id
    )
    .fetch_all(pool)
    .await?;
    Ok(credentials)
  }

  pub async fn from_session_id(
    pool: &PgPool,
    session_id: i64,
  ) -> Result<UserSession, Box<dyn Error>> {
    let session = sqlx::query_as!(
      UserSession,
      r#"
        SELECT 
          session_id, user_id, refresh_hash, webauthn_id
        FROM user_sessions WHERE session_id = $1
      "#,
      session_id
    )
    .fetch_one(pool)
    .await?;
    Ok(session)
  }

  pub async fn create_session(
    pool: &PgPool,
    user_id: i32,
    webauthn_id: i32,
  ) -> Result<(String, UserSession), Box<dyn Error>> {
    // NOTE: if we ever support concurrent servers in the future, we need to pass an "instance ID"
    // from an environment variable in here to avoid conflicts.
    let mut session_id_generator = snowflaked::Generator::new(0);
    let session_id = session_id_generator.generate::<i64>();

    let refresh_token = Alphanumeric.sample_string(&mut rand::thread_rng(), 64);
    let refresh_token_cloned = refresh_token.clone();
    let refresh_salt = SaltString::generate(&mut argon2::password_hash::rand_core::OsRng);
    let refresh_hash = spawn_blocking(move || {
      Argon2::default()
        .hash_password(refresh_token_cloned.as_bytes(), &refresh_salt)
        .map(|x| x.to_string())
    })
    .await??;

    let session = UserSession {
      session_id,
      user_id,
      refresh_hash,
      webauthn_id,
    };

    sqlx::query!(
      r#"
        INSERT INTO user_sessions(session_id, user_id, refresh_hash, webauthn_id)
        VALUES ($1, $2, $3, $4)
      "#,
      session.session_id,
      session.user_id,
      session.refresh_hash,
      session.webauthn_id
    )
    .execute(pool)
    .await?;

    Ok((refresh_token, session))
  }

  pub async fn refresh_session(&mut self, pool: &PgPool) -> Result<String, Box<dyn Error>> {
    let refresh_token = Alphanumeric.sample_string(&mut rand::thread_rng(), 64);
    let refresh_token_cloned = refresh_token.clone();
    let refresh_salt = SaltString::generate(&mut argon2::password_hash::rand_core::OsRng);
    let refresh_hash = spawn_blocking(move || {
      Argon2::default()
        .hash_password(refresh_token_cloned.as_bytes(), &refresh_salt)
        .map(|x| x.to_string())
    })
    .await??;

    sqlx::query!(
      r#"
        UPDATE user_sessions SET refresh_hash = $1 WHERE session_id = $2
      "#,
      refresh_hash,
      self.session_id
    )
    .execute(pool)
    .await?;

    self.refresh_hash = refresh_hash;

    Ok(refresh_token)
  }

  pub async fn delete_session(&mut self, pool: &PgPool) -> Result<(), Box<dyn Error>> {
    sqlx::query!(
      r#"
        DELETE FROM user_sessions WHERE session_id = $1
      "#,
      self.session_id
    )
    .execute(pool)
    .await?;
    Ok(())
  }
  /*
    pub async fn update(&self, pool: &PgPool) -> Result<(), Box<dyn Error>> {
      sqlx::query!(
        r#"
        UPDATE users SET email=$1, username=$2, name=$3, is_suspended=$4, credential_uuid=$5
        WHERE id=$6
      "#,
        self.email, self.username, self.name, self.is_suspended, self.credential_uuid, self.id
      ).execute(pool).await?;
      Ok(())
    }
  */
}
