use axum::{
  Extension, Json, Router,
  extract::State,
  routing::{get, post},
};
use serde::{Deserialize, Serialize};

use crate::{
  AppState,
  auth::identity::IdentityAccessClaims,
  user::{AdminCtx, User},
};

pub mod credential;
pub mod identity;
pub mod login;
pub mod register;
pub mod session;

#[derive(Serialize, Deserialize)]
struct TestResponse {
  pub user: Option<User>,
}

async fn test_auth_route(
  State(app_state): State<AppState>,
  authenticated_user: Option<Extension<IdentityAccessClaims>>,
) -> Json<TestResponse> {
  let user_id = match authenticated_user {
    Some(u) => u.user_id,
    None => 0,
  };
  match User::from_user_id(&app_state.pool, user_id).await {
    Ok(u) => Json(TestResponse { user: Some(u) }),
    Err(_) => Json(TestResponse { user: None }),
  }
}

async fn test_admin_auth_route(admin_ctx: AdminCtx) -> Json<TestResponse> {
  Json(TestResponse {
    user: Some(admin_ctx.user),
  })
}

pub fn router() -> Router<crate::AppState> {
  Router::new()
    .route("/v1/auth/test", get(test_auth_route))
    .route("/v1/auth/admin-test", get(test_admin_auth_route))
    .route(
      "/v1/auth/register/passkey/initiate",
      post(register::start_passkey_registration),
    )
    .route(
      "/v1/auth/register/passkey/finalize",
      post(register::finish_passkey_registration),
    )
    .route(
      "/v1/auth/login/passkey/initiate",
      post(login::start_passkey_login),
    )
    .route(
      "/v1/auth/login/passkey/finalize",
      post(login::finish_passkey_login),
    )
    .route("/v1/auth/refresh", post(identity::refresh_auth))
    .route("/v1/auth/logout", post(identity::logout_current_session))
}
