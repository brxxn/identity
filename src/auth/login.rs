use std::time::{SystemTime, UNIX_EPOCH};

use axum::{Json, extract::State};
use base64::{Engine, prelude::BASE64_STANDARD};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use sqlx::types::Uuid;
use webauthn_rs::prelude::{
  DiscoverableAuthentication, DiscoverableKey, Passkey, PublicKeyCredential,
  RequestChallengeResponse,
};

use crate::{
  auth::{
    credential::WebauthnCredential,
    identity::{IdentityAccessClaims, IdentityRefreshClaims},
    session::UserSession,
  },
  response::{ApiErr, ApiResponse},
  user::User,
};

#[derive(Serialize, Deserialize)]
pub struct SignedLoginChallengeClaims {
  pub iat: u64,
  pub exp: u64,
  pub auth: DiscoverableAuthentication,
}

#[derive(Serialize)]
pub struct LoginInitiateResponse {
  pub challenge_signature: String,
  pub challenge_response: RequestChallengeResponse,
}

#[derive(Deserialize)]
pub struct LoginFinalizeRequest {
  pub challenge_signature: String,
  pub pk_credential: PublicKeyCredential,
}

#[derive(Serialize)]
pub struct LoginFinalizeResponse {
  pub access_token: String,
  pub refresh_token: String,
  pub session: UserSession,
  pub credential: WebauthnCredential,
  pub user: User,
}

impl SignedLoginChallengeClaims {
  fn to_token(&self, state: &crate::AppState) -> String {
    let encoding_key =
      &EncodingKey::from_secret(state.private_keys.passkey_registration_key.as_bytes());
    jsonwebtoken::encode(&Header::default(), &self, encoding_key).expect("Failed to encode key!")
  }

  fn from_token(token: String, state: &crate::AppState) -> Option<SignedLoginChallengeClaims> {
    let decoded_key =
      DecodingKey::from_secret(state.private_keys.passkey_registration_key.as_bytes());
    let decoded_token = match jsonwebtoken::decode::<SignedLoginChallengeClaims>(
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

pub async fn start_passkey_login(
  State(state): State<crate::AppState>,
) -> ApiResponse<LoginInitiateResponse> {
  let iat = SystemTime::now()
    .duration_since(UNIX_EPOCH)
    .expect("Time went backwards lol")
    .as_secs();

  let Ok((rcr, pka)) = state.webauthn.start_discoverable_authentication() else {
    return ApiResponse::Err(ApiErr::InternalServerError);
  };

  let signed_claims = SignedLoginChallengeClaims {
    iat,
    exp: iat + 330,
    auth: pka,
  };

  ApiResponse::Ok(LoginInitiateResponse {
    challenge_signature: signed_claims.to_token(&state),
    challenge_response: rcr,
  })
}

pub async fn finish_passkey_login(
  State(state): State<crate::AppState>,
  Json(payload): Json<LoginFinalizeRequest>,
) -> ApiResponse<LoginFinalizeResponse> {
  let Some(signed_challenge) =
    SignedLoginChallengeClaims::from_token(payload.challenge_signature, &state)
  else {
    return ApiResponse::Err(ApiErr::InvalidChallenge);
  };

  let Some(credential_handle) = payload.pk_credential.get_user_unique_id() else {
    return ApiResponse::Err(ApiErr::InvalidCredential);
  };

  let Ok(credential_uuid) = Uuid::from_slice(credential_handle) else {
    return ApiResponse::Err(ApiErr::InvalidCredential);
  };

  let Ok(credential_vec) =
    WebauthnCredential::from_credential_uuid(&state.pool, credential_uuid).await
  else {
    return ApiResponse::Err(ApiErr::InternalServerError);
  };

  let Ok(user) = User::from_credential_uuid(&state.pool, &credential_uuid).await else {
    return ApiResponse::Err(ApiErr::UserDeleted);
  };

  if user.is_suspended {
    return ApiResponse::Err(ApiErr::UserSuspended);
  }

  let credentials = credential_vec
    .iter()
    .map(|x| serde_json::from_str::<Passkey>(&x.serialized_passkey))
    .filter_map(Result::ok)
    .map(DiscoverableKey::from)
    .collect::<Vec<DiscoverableKey>>();

  match state.webauthn.finish_discoverable_authentication(
    &payload.pk_credential,
    signed_challenge.auth,
    &credentials,
  ) {
    Ok(result) => {
      let possible_credentials = credential_vec
        .into_iter()
        .filter(|x| {
          BASE64_STANDARD
            .decode(&x.credential_id)
            .is_ok_and(|y| &y == result.cred_id())
        })
        .collect::<Vec<WebauthnCredential>>();

      let Some(credential) = possible_credentials.first() else {
        return ApiResponse::Err(ApiErr::InternalServerError);
      };

      let Ok((refresh_token, session)) =
        UserSession::create_session(&state.pool, user.id, credential.id).await
      else {
        return ApiResponse::Err(ApiErr::InternalServerError);
      };

      let access_claims =
        IdentityAccessClaims::create_from_passkey(&user, credential.id, session.session_id);

      let refresh_claims = IdentityRefreshClaims {
        session_id: session.session_id,
        refresh_token,
      };

      ApiResponse::Ok(LoginFinalizeResponse {
        access_token: access_claims.to_token(&state),
        refresh_token: refresh_claims.to_jwt(&state),
        credential: credential.clone(),
        user,
        session,
      })
    }
    Err(_) => ApiResponse::Err(ApiErr::Other(
      "webauthn_error".to_string(),
      "An unexpected webauthn passkey registration error occurred.".to_string(),
    )),
  }
}
