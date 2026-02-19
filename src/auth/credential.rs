use std::error::Error;

use serde::{Deserialize, Serialize};
use sqlx::{PgPool, types::Uuid};

#[derive(Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct WebauthnCredential {
  pub id: i32,
  pub name: String,
  #[serde(skip)]
  pub credential_uuid: Uuid,
  pub credential_id: String,
  #[serde(skip)]
  pub serialized_passkey: String,
}

impl WebauthnCredential {
  pub async fn from_credential_uuid(
    pool: &PgPool,
    credential_uuid: Uuid,
  ) -> Result<Vec<WebauthnCredential>, Box<dyn Error>> {
    let credentials = sqlx::query_as!(
      WebauthnCredential,
      r#"
        SELECT 
          id, name, credential_id, credential_uuid, serialized_passkey
        FROM user_webauthn_credentials WHERE credential_uuid = $1
      "#,
      credential_uuid
    )
    .fetch_all(pool)
    .await?;
    Ok(credentials)
  }

  pub async fn create(&mut self, pool: &PgPool) -> Result<&WebauthnCredential, Box<dyn Error>> {
    let result = sqlx::query_scalar!(
      r#"
        INSERT INTO user_webauthn_credentials(name, credential_uuid, credential_id, serialized_passkey) VALUES 
          ($1, $2, $3, $4) RETURNING id
      "#,
      self.name, self.credential_uuid, self.credential_id, self.serialized_passkey
    ).fetch_one(pool).await?;
    self.id = result;
    Ok(self)
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
