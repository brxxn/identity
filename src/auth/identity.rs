use std::{
  collections::HashSet,
  time::{SystemTime, UNIX_EPOCH},
};

use argon2::{Argon2, PasswordHash, PasswordVerifier};
use axum::{Extension, Json, extract::State};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use serde_with::{DisplayFromStr, serde_as};

use crate::{
  AppState,
  auth::session::UserSession,
  response::{ApiErr, ApiResponse, EmptyResponse},
  user::User,
};

#[serde_as]
#[derive(Clone, Serialize, Deserialize)]
pub struct IdentityAccessClaims {
  pub user_id: i32,
  pub method: String,
  pub iat: u64,
  pub exp: u64,
  // These values are mostly for the client, server should still check
  pub email: String,
  pub username: String,
  pub name: String,
  pub webauthn_id: i32,
  /// This is EXCLUSIVELY for client use (to avoid making api calls to backend
  /// to check admin). Do NOT use it server-side because it may not be up to
  /// date!
  pub is_admin: bool,
  #[serde_as(as = "DisplayFromStr")]
  pub session_id: i64,
}

/// The purpose of putting the refresh token in a JWT is less about the
/// security and more about forcing the session ID to be kept with the
/// refresh token, since we don't lookup by refresh token
#[serde_as]
#[derive(Clone, Serialize, Deserialize)]
pub struct IdentityRefreshClaims {
  #[serde_as(as = "DisplayFromStr")]
  pub session_id: i64,
  pub refresh_token: String,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct RefreshTokenRequest {
  pub refresh_token: String,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct RefreshTokenResponse {
  pub access_token: String,
  pub refresh_token: String,
}

pub fn authenticate_jwt(token: String, state: &AppState) -> Option<IdentityAccessClaims> {
  let decoded_key =
    &DecodingKey::from_secret(state.private_keys.identity_access_jwt_key.as_bytes());
  let decoded_token = jsonwebtoken::decode::<IdentityAccessClaims>(
    &token,
    &decoded_key,
    &Validation::new(jsonwebtoken::Algorithm::HS256),
  )
  .ok()?;
  Some(decoded_token.claims)
}

impl IdentityAccessClaims {
  pub fn create_from_passkey(
    user: &User,
    webauthn_id: i32,
    session_id: i64,
  ) -> IdentityAccessClaims {
    let iat = SystemTime::now()
      .duration_since(UNIX_EPOCH)
      .expect("Time went backwards lol")
      .as_secs();

    IdentityAccessClaims {
      user_id: user.id,
      method: "passkey".to_string(),
      iat,
      exp: iat + 3600,
      email: user.email.clone(),
      username: user.username.clone(),
      name: user.name.clone(),
      is_admin: user.is_admin,
      webauthn_id,
      session_id,
    }
  }

  pub fn to_token(&self, state: &AppState) -> String {
    let encoding_key =
      &EncodingKey::from_secret(state.private_keys.identity_access_jwt_key.as_bytes());
    jsonwebtoken::encode(&Header::default(), &self, encoding_key).expect("Failed to encode key!")
  }
}

impl IdentityRefreshClaims {
  pub fn to_jwt(&self, state: &AppState) -> String {
    let encoding_key =
      &EncodingKey::from_secret(state.private_keys.identity_refresh_jwt_key.as_bytes());
    jsonwebtoken::encode(&Header::default(), &self, encoding_key).expect("Failed to encode key!")
  }

  pub fn from_jwt(jwt: String, state: &AppState) -> Option<IdentityRefreshClaims> {
    let decoded_key =
      &DecodingKey::from_secret(state.private_keys.identity_refresh_jwt_key.as_bytes());
    // refesh tokens currently don't expire, so we disable this validation
    let mut validation = Validation::new(jsonwebtoken::Algorithm::HS256);
    validation.validate_exp = false;
    validation.required_spec_claims = HashSet::new();
    let decoded_token =
      jsonwebtoken::decode::<IdentityRefreshClaims>(&jwt, &decoded_key, &validation).ok()?;
    Some(decoded_token.claims)
  }
}

pub async fn refresh_auth(
  State(state): State<crate::AppState>,
  Json(payload): Json<RefreshTokenRequest>,
) -> ApiResponse<RefreshTokenResponse> {
  let Some(refresh_claims) = IdentityRefreshClaims::from_jwt(payload.refresh_token, &state) else {
    tracing::info!("refresh claims not valid!");
    return ApiResponse::Err(ApiErr::SessionExpired);
  };

  let Ok(mut session) = UserSession::from_session_id(&state.pool, refresh_claims.session_id).await
  else {
    tracing::info!("session id lookup failure");
    return ApiResponse::Err(ApiErr::SessionExpired);
  };

  let Ok(refresh_hash) = PasswordHash::new(&session.refresh_hash) else {
    return ApiResponse::Err(ApiErr::InternalServerError);
  };

  if !Argon2::default()
    .verify_password(refresh_claims.refresh_token.as_bytes(), &refresh_hash)
    .is_ok()
  {
    tracing::info!("refresh token not valid!");
    return ApiResponse::Err(ApiErr::SessionExpired);
  }

  let Ok(refresh_token) = session.refresh_session(&state.pool).await else {
    return ApiResponse::Err(ApiErr::InternalServerError);
  };

  let Ok(user) = User::from_user_id(&state.pool, session.user_id).await else {
    return ApiResponse::Err(ApiErr::UserDeleted);
  };

  if user.is_suspended {
    return ApiResponse::Err(ApiErr::UserSuspended);
  }

  let access_token =
    IdentityAccessClaims::create_from_passkey(&user, session.webauthn_id, session.session_id);

  let jwt_refresh_claims = IdentityRefreshClaims {
    session_id: session.session_id,
    refresh_token,
  };
  let jwt_refresh_token = jwt_refresh_claims.to_jwt(&state);

  ApiResponse::Ok(RefreshTokenResponse {
    access_token: access_token.to_token(&state),
    refresh_token: jwt_refresh_token,
  })
}

pub async fn logout_current_session(
  State(state): State<crate::AppState>,
  _: User,
  Extension(claims): Extension<IdentityAccessClaims>,
) -> ApiResponse<EmptyResponse> {
  let Ok(mut session) = UserSession::from_session_id(&state.pool, claims.session_id).await else {
    // they probably already logged out but still have a valid access token, so just
    // tell them they're already logged out.
    return ApiResponse::EmptyOk;
  };

  if !session.delete_session(&state.pool).await.is_ok() {
    return ApiResponse::Err(ApiErr::InternalServerError);
  }

  ApiResponse::EmptyOk
}
