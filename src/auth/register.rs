use std::time::{SystemTime, UNIX_EPOCH};

use axum::{Json, extract::State};
use base64::{Engine, prelude::BASE64_STANDARD};
use base64urlsafedata::HumanBinaryData;
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use sqlx::types::Uuid;
use webauthn_rs::prelude::{
  CreationChallengeResponse, PasskeyRegistration, RegisterPublicKeyCredential,
};

use crate::{
  auth::credential::WebauthnCredential,
  response::{ApiErr, ApiResponse},
  user::User,
};

#[derive(Serialize, Deserialize)]
pub struct RegistrationClaims {
  pub user_id: i32,
  pub iat: u64,
  pub exp: u64,
  // These values are mostly for the client, server should still check
  pub email: String,
  pub username: String,
  pub name: String,
}

#[derive(Serialize, Deserialize)]
pub struct SignedChallengeClaims {
  pub credential_uuid: Uuid,
  pub iat: u64,
  pub exp: u64,
  pub reg: PasskeyRegistration,
}

#[derive(Deserialize)]
pub struct RegistrationInitiateRequest {
  pub registration_token: String,
}

#[derive(Serialize)]
pub struct RegistrationInitiateResponse {
  pub challenge_signature: String,
  pub challenge_response: CreationChallengeResponse,
}

#[derive(Deserialize)]
pub struct RegistrationFinalizeRequest {
  pub challenge_signature: String,
  pub registration_token: String,
  pub pk_credential: RegisterPublicKeyCredential,
}

impl RegistrationClaims {
  pub fn new(user: &User) -> Self {
    let iat = SystemTime::now()
      .duration_since(UNIX_EPOCH)
      .expect("Time went backwards lol")
      .as_secs();
    RegistrationClaims {
      user_id: user.id,
      iat,
      exp: iat + 86400,
      email: user.email.clone(),
      username: user.username.clone(),
      name: user.name.clone()
    }
  }

  pub fn from_token(token: String, state: &crate::AppState) -> Option<RegistrationClaims> {
    let decoded_key = &DecodingKey::from_secret(state.private_keys.registration_jwt_key.as_bytes());
    let decoded_token = match jsonwebtoken::decode::<RegistrationClaims>(
      &token,
      &decoded_key,
      &Validation::new(jsonwebtoken::Algorithm::HS256),
    ) {
      Ok(t) => t,
      Err(e) => {
        tracing::warn!("Failed to decode registration token: {e}");
        return None;
      }
    };
    Some(decoded_token.claims)
  }

  // TODO: remove this when i actually use it
  #[allow(dead_code)]
  pub fn to_token(&self, state: &crate::AppState) -> String {
    let encoding_key =
      &EncodingKey::from_secret(state.private_keys.registration_jwt_key.as_bytes());
    jsonwebtoken::encode(&Header::default(), &self, encoding_key).expect("Failed to encode key!")
  }
}

impl SignedChallengeClaims {
  fn to_token(&self, state: &crate::AppState) -> String {
    let encoding_key =
      &EncodingKey::from_secret(state.private_keys.passkey_registration_key.as_bytes());
    jsonwebtoken::encode(&Header::default(), &self, encoding_key).expect("Failed to encode key!")
  }

  fn from_token(token: String, state: &crate::AppState) -> Option<SignedChallengeClaims> {
    let decoded_key =
      DecodingKey::from_secret(state.private_keys.passkey_registration_key.as_bytes());
    let decoded_token = match jsonwebtoken::decode::<SignedChallengeClaims>(
      &token,
      &decoded_key,
      &Validation::new(jsonwebtoken::Algorithm::HS256),
    ) {
      Ok(t) => t,
      Err(e) => {
        tracing::warn!("Failed to decode signed challenge token: {e}");
        return None;
      }
    };
    Some(decoded_token.claims)
  }
}

pub async fn start_passkey_registration(
  State(state): State<crate::AppState>,
  Json(payload): Json<RegistrationInitiateRequest>,
) -> ApiResponse<RegistrationInitiateResponse> {
  let Some(registration) = RegistrationClaims::from_token(payload.registration_token, &state)
  else {
    return ApiResponse::Err(ApiErr::ExpiredRegistration);
  };

  let Ok(user) = User::from_user_id(&state.pool, registration.user_id).await else {
    return ApiResponse::Err(ApiErr::UserDeleted);
  };

  if user.is_suspended {
    return ApiResponse::Err(ApiErr::UserSuspended);
  }

  if user.email != registration.email {
    return ApiResponse::Err(ApiErr::Other(
      "email_changed".to_string(),
      "The email associated with this account has changed, so this link is no longer valid."
        .to_string(),
    ));
  }

  let Ok(credential_vec) =
    WebauthnCredential::from_credential_uuid(&state.pool, user.credential_uuid).await
  else {
    return ApiResponse::Err(ApiErr::InternalServerError);
  };

  let exclude_credentials = credential_vec
    .into_iter()
    .map(|x| BASE64_STANDARD.decode(x.credential_id))
    .filter_map(Result::ok)
    .map(HumanBinaryData::from)
    .collect();

  let Ok((ccr, pkr)) = state.webauthn.start_passkey_registration(
    user.credential_uuid,
    &user.username,
    &user.name,
    Some(exclude_credentials),
  ) else {
    return ApiResponse::Err(ApiErr::Other(
      "webauthn_error".to_string(),
      "An unexpected webauthn passkey registration error occurred.".to_string(),
    ));
  };

  let iat = SystemTime::now()
    .duration_since(UNIX_EPOCH)
    .expect("Time went backwards lol")
    .as_secs();

  let signed_claims = SignedChallengeClaims {
    credential_uuid: user.credential_uuid,
    iat,
    exp: iat + 330,
    reg: pkr,
  };

  ApiResponse::Ok(RegistrationInitiateResponse {
    challenge_signature: signed_claims.to_token(&state),
    challenge_response: ccr,
  })
}

pub async fn finish_passkey_registration(
  State(state): State<crate::AppState>,
  Json(payload): Json<RegistrationFinalizeRequest>,
) -> ApiResponse<RegistrationInitiateResponse> {
  let Some(registration) = RegistrationClaims::from_token(payload.registration_token, &state)
  else {
    return ApiResponse::Err(ApiErr::ExpiredRegistration);
  };

  let Some(signed_challenge) =
    SignedChallengeClaims::from_token(payload.challenge_signature, &state)
  else {
    return ApiResponse::Err(ApiErr::InvalidChallenge);
  };

  let Ok(user) = User::from_user_id(&state.pool, registration.user_id).await else {
    return ApiResponse::Err(ApiErr::UserDeleted);
  };

  if user.is_suspended {
    return ApiResponse::Err(ApiErr::UserSuspended);
  }

  if signed_challenge.credential_uuid != user.credential_uuid {
    return ApiResponse::Err(ApiErr::InvalidChallenge);
  }

  if user.email != registration.email {
    return ApiResponse::Err(ApiErr::Other(
      "email_changed".to_string(),
      "The email associated with this account has changed, so this link is no longer valid."
        .to_string(),
    ));
  }

  let Ok(credential_vec) =
    WebauthnCredential::from_credential_uuid(&state.pool, user.credential_uuid).await
  else {
    return ApiResponse::Err(ApiErr::InternalServerError);
  };

  for credential in credential_vec {
    if credential.credential_id == payload.pk_credential.id {
      return ApiResponse::Err(ApiErr::Other(
        "credential_already_registered".to_string(),
        "This credential is already registered!".to_string(),
      ));
    }
  }

  match state
    .webauthn
    .finish_passkey_registration(&payload.pk_credential, &signed_challenge.reg)
  {
    Ok(reg) => {
      let mut db_cred = WebauthnCredential {
        id: 0,
        name: "Unnamed Passkey".to_string(),
        credential_id: BASE64_STANDARD.encode(reg.cred_id()),
        credential_uuid: user.credential_uuid,
        serialized_passkey: serde_json::to_string(&reg).expect("Failed to serialize passkey"),
      };

      let Ok(_) = db_cred.create(&state.pool).await else {
        return ApiResponse::Err(ApiErr::InternalServerError);
      };

      ApiResponse::EmptyOk
    }
    Err(_) => ApiResponse::Err(ApiErr::Other(
      "webauthn_error".to_string(),
      "An unexpected webauthn passkey registration error occurred.".to_string(),
    )),
  }
}
