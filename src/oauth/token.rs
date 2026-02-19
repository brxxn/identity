use std::error::Error;

use rand::distributions::{Alphanumeric, DistString};
use redis::AsyncCommands;
use serde::{Deserialize, Serialize};

use crate::AppState;

#[derive(Clone, Serialize, Deserialize)]
pub struct OauthAccessTokenData {
  pub user_id: i32,
  pub client_id: String,
  pub nonce: Option<String>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct OauthRefreshTokenData {
  pub user_id: i32,
  pub client_id: String,
  pub nonce: Option<String>,
}

impl OauthAccessTokenData {
  pub async fn from_token(state: &AppState, token: String) -> Result<Option<OauthAccessTokenData>, Box<dyn Error>> {
    let key = format!("oauth_access_token:{}", token);
    let token_data: Option<String> = state.redis_connection.clone().get(key).await?;
    match token_data {
      Some(data) => Ok(Some(serde_json::from_str::<OauthAccessTokenData>(data.as_str())?)),
      None => Ok(None)
    }
  }

  pub async fn save_to_token(&self, state: &AppState) -> Result<String, Box<dyn Error>> {
    let oauth_token = Alphanumeric.sample_string(&mut rand::thread_rng(), 64);
    let key = format!("oauth_access_token:{}", oauth_token);
    let value = serde_json::to_string(self)?;
    let _: () = state.redis_connection.clone().set_ex(key, value, 3600).await?;
    Ok(oauth_token)
  }
}

impl OauthRefreshTokenData {
  pub async fn from_token(state: &AppState, token: String) -> Result<Option<OauthAccessTokenData>, Box<dyn Error>> {
    let key = format!("oauth_refresh_token:{}", token);
    let token_data: Option<String> = state.redis_connection.clone().get(key).await?;
    match token_data {
      Some(data) => Ok(Some(serde_json::from_str::<OauthAccessTokenData>(data.as_str())?)),
      None => Ok(None)
    }
  }

  pub async fn save_to_token(&self, state: &AppState) -> Result<String, Box<dyn Error>> {
    let oauth_token = Alphanumeric.sample_string(&mut rand::thread_rng(), 64);
    let key = format!("oauth_refresh_token:{}", oauth_token);
    let value = serde_json::to_string(self)?;
    let _: () = state.redis_connection.clone().set_ex(key, value, 1209600).await?;
    Ok(oauth_token)
  }
}

