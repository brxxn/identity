use std::error::Error;

use rand::distributions::{Alphanumeric, DistString};
use redis::AsyncCommands;
use serde::{Deserialize, Serialize};

use crate::AppState;

#[derive(Clone, Serialize, Deserialize)]
pub struct OauthCodeData {
  pub user_id: i32,
  pub client_id: String,
  pub nonce: Option<String>,
  pub redirect_uri: String,
}

impl OauthCodeData {
  pub async fn from_code(state: &AppState, code: String) -> Result<Option<OauthCodeData>, Box<dyn Error>> {
    let key = format!("oauth_code:{}", code);
    let code_data: Option<String> = state.redis_connection.clone().get(key).await?;
    match code_data {
      Some(data) => {
        tracing::info!("found oauth code!");
        Ok(Some(serde_json::from_str::<OauthCodeData>(data.as_str())?))
      },
      None => Ok(None)
    }
  }

  pub async fn save_to_code(&self, state: &AppState) -> Result<String, Box<dyn Error>> {
    let oauth_code = Alphanumeric.sample_string(&mut rand::thread_rng(), 64);
    let key = format!("oauth_code:{}", oauth_code);
    let value = serde_json::to_string(self)?;
    let _: () = state.redis_connection.clone().set_ex(key, value, 300).await?;
    Ok(oauth_code)
  }
}