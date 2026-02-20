use std::error::Error;

use axum::{
  Router,
  routing::{get, post},
};
use jsonwebtoken::{EncodingKey, Header};
use rsa::pkcs8::EncodePrivateKey;
use serde::Serialize;
use serde_with::skip_serializing_none;

use crate::{
  AppState, client::IdentityClient, group::IdentityGroup,
  oauth::authorization::UserAppAuthorization, user::User,
};

pub mod authorization;
pub mod code;
pub mod routes;
pub mod token;
pub mod wellknown;

#[skip_serializing_none]
#[derive(Serialize)]
pub struct OidcIdTokenClaims {
  pub iss: String,
  pub sub: String,
  pub aud: String,
  pub exp: u64,
  pub iat: u64,
  // TODO: this will eventually return something that is not a fake value!
  pub auth_time: u64,
  pub nonce: Option<String>,
  pub name: String,
  pub preferred_username: String,
  pub email: String,
  pub email_verified: bool,
  pub groups: Vec<String>,
  pub roles: Vec<String>,
}

pub async fn create_id_token(
  state: &AppState,
  user: &User,
  client: &IdentityClient,
  groups: Vec<IdentityGroup>,
  nonce: Option<String>,
  authorization: &UserAppAuthorization,
) -> Result<String, Box<dyn Error>> {
  let Some(kid) = state.private_keys.oidc_jwt_keys.keys().max() else {
    panic!("No JWT keys are loaded!");
  };

  let iat = std::time::SystemTime::now()
    .duration_since(std::time::SystemTime::UNIX_EPOCH)
    .expect("time has somehow gone backwards...")
    .as_secs();

  let roles = client.get_user_roles(&state.pool, user, &groups).await?;
  let groups = groups
    .iter()
    .map(|x| x.slug.clone())
    .collect::<Vec<String>>();

  let claims = OidcIdTokenClaims {
    iss: state.oidc_issuer_uri.clone(),
    sub: authorization.sub.clone(),
    aud: client.client_id.clone(),
    iat,
    exp: iat + 3600,
    auth_time: iat,
    nonce,
    name: user.name.clone(),
    preferred_username: user.username.clone(),
    email: user.email.clone(),
    email_verified: true,
    groups,
    roles,
  };

  let private_key = state.private_keys.oidc_jwt_keys.get(kid).unwrap();
  let private_key_pem = private_key
    .to_pkcs8_pem(rsa::pkcs8::LineEnding::LF)
    .unwrap();

  let encoding_key = &EncodingKey::from_rsa_pem(private_key_pem.as_bytes()).unwrap();
  let mut key_header = Header::new(jsonwebtoken::Algorithm::RS256);
  key_header.kid = Some(kid.to_string());
  Ok(
    jsonwebtoken::encode(&key_header, &claims, encoding_key)
      .expect("failed to encode OIDC id_token"),
  )
}

pub fn router() -> Router<AppState> {
  Router::new()
    .route(
      "/v1/oauth/authorize/preview",
      post(routes::oauth_authorize_preview),
    )
    .route(
      "/v1/oauth/authorize/approve",
      post(routes::oauth_authorize_approve),
    )
    .route("/v1/oauth/token", post(routes::oauth_token))
    .route("/v1/oauth/userinfo", get(routes::oauth_userinfo))
    .route(
      "/.well-known/openid-configuration",
      get(wellknown::openid_configuration),
    )
    .route("/.well-known/jwks", get(wellknown::jwks))
}
