use std::error::Error;

use axum::{
  Router,
  extract::{FromRef, FromRequestParts},
  routing::{get, post},
};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use webauthn_rs::prelude::Url;

use crate::{
  AppState,
  auth::{identity::IdentityAccessClaims, register::RegistrationClaims},
  group::IdentityGroup,
  response::{ApiErr, ApiResponse, EmptyResponse}, smtp::{new_registration_message, send_mail},
};

pub mod routes;

#[derive(Serialize, Deserialize, sqlx::FromRow)]
pub struct User {
  pub id: i32,
  pub email: String,
  pub username: String,
  pub name: String,
  pub is_suspended: bool,
  pub credential_uuid: sqlx::types::Uuid,
  pub is_admin: bool,
}

/// This should be extracted in routes where admin is required INSTEAD of
/// extracting User. It will check admin for you.
pub struct AdminCtx {
  pub user: User,
}

impl User {
  /// This will probably be deprecated whenever I feel like adding pagination and
  /// if some other person decides to actually use this.
  pub async fn list_all_users(pool: &PgPool) -> Result<Vec<User>, Box<dyn Error>> {
    let users = sqlx::query_as!(
      User,
      r#"
        SELECT id, email, username, name, is_suspended, credential_uuid, is_admin FROM users
      "#
    )
    .fetch_all(pool)
    .await?;
    Ok(users)
  }

  pub async fn from_user_id(pool: &PgPool, user_id: i32) -> Result<User, Box<dyn Error>> {
    let user = sqlx::query_as!(
      User,
      r#"
        SELECT id, email, username, name, is_suspended, credential_uuid, is_admin FROM users WHERE id = $1
      "#,
      user_id
    ).fetch_one(pool).await?;
    Ok(user)
  }

  pub async fn from_credential_uuid(
    pool: &PgPool,
    cred_uuid: &sqlx::types::Uuid,
  ) -> Result<User, Box<dyn Error>> {
    let user = sqlx::query_as!(
      User,
      r#"
        SELECT id, email, username, name, is_suspended, credential_uuid, is_admin FROM users WHERE credential_uuid = $1
      "#,
      cred_uuid
    ).fetch_one(pool).await?;
    Ok(user)
  }

  pub async fn create(&mut self, pool: &PgPool) -> Result<&User, Box<dyn Error>> {
    let result = sqlx::query_scalar!(
      r#"
        INSERT INTO users(email, username, name, is_suspended, credential_uuid, is_admin) VALUES 
          ($1, $2, $3, $4, $5, $6) RETURNING id
      "#,
      self.email,
      self.username,
      self.name,
      self.is_suspended,
      self.credential_uuid,
      self.is_admin
    )
    .fetch_one(pool)
    .await?;
    self.id = result;
    Ok(self)
  }

  pub async fn update(&self, pool: &PgPool) -> Result<(), Box<dyn Error>> {
    sqlx::query!(
      r#"
        UPDATE users SET email=$1, username=$2, name=$3, is_suspended=$4, credential_uuid=$5, is_admin=$6
        WHERE id=$7
      "#,
      self.email, self.username, self.name, self.is_suspended, self.credential_uuid, self.is_admin, self.id
    ).execute(pool).await?;
    Ok(())
  }

  pub async fn get_groups(&self, pool: &PgPool) -> Result<Vec<IdentityGroup>, Box<dyn Error>> {
    let results = sqlx::query_as!(
      IdentityGroup,
      r#"
        SELECT g.* FROM permission_groups g
        JOIN permission_group_membership m
        ON g.id = m.group_id
        WHERE m.user_id = $1
      "#,
      self.id
    )
    .fetch_all(pool)
    .await?;
    Ok(results)
  }

  pub async fn send_registration_mail(&self, state: &AppState) -> Result<(), Box<dyn Error>> {
    let claims = RegistrationClaims::new(self);
    let token = claims.to_token(state);
    // TODO: use webauthn instead of OIDC issuer uri
    let registration_link = format!("{}/auth/register/passkey?t={}", state.oidc_issuer_uri.clone(), token);
    let registration_url = Url::parse(&registration_link)?;
    let origin = registration_url.host().unwrap();

    let message = new_registration_message(self, registration_link, origin.to_string());

    send_mail(state, message).await
  }
}

impl<S> FromRequestParts<S> for User
where
  AppState: FromRef<S>,
  S: Send + Sync,
{
  type Rejection = ApiResponse<EmptyResponse>;

  async fn from_request_parts(
    parts: &mut http::request::Parts,
    state: &S,
  ) -> Result<Self, Self::Rejection> {
    let Some(claims) = parts.extensions.get::<IdentityAccessClaims>() else {
      return Err(ApiResponse::Err(ApiErr::LoginRequired));
    };

    let app_state = AppState::from_ref(state);

    let Ok(user) = User::from_user_id(&app_state.pool, claims.user_id).await else {
      return Err(ApiResponse::Err(ApiErr::UserDeleted));
    };

    if user.is_suspended {
      return Err(ApiResponse::Err(ApiErr::UserSuspended));
    }

    Ok(user)
  }
}

impl<S> FromRequestParts<S> for AdminCtx
where
  AppState: FromRef<S>,
  S: Send + Sync,
{
  type Rejection = ApiResponse<EmptyResponse>;

  async fn from_request_parts(
    parts: &mut http::request::Parts,
    state: &S,
  ) -> Result<Self, Self::Rejection> {
    let Some(claims) = parts.extensions.get::<IdentityAccessClaims>() else {
      return Err(ApiResponse::Err(ApiErr::LoginRequired));
    };

    let app_state = AppState::from_ref(state);

    let Ok(user) = User::from_user_id(&app_state.pool, claims.user_id).await else {
      return Err(ApiResponse::Err(ApiErr::UserDeleted));
    };

    if user.is_suspended {
      return Err(ApiResponse::Err(ApiErr::UserSuspended));
    }

    if !user.is_admin {
      return Err(ApiResponse::Err(ApiErr::AdminRequired));
    }

    Ok(AdminCtx { user })
  }
}

pub fn router() -> Router<AppState> {
  Router::new()
    .route(
      "/v1/users",
      get(routes::list_users).post(routes::create_user),
    )
    .route(
      "/v1/users/{user_id}",
      get(routes::get_user_by_id).patch(routes::update_user),
    )
    .route(
      "/v1/users/{user_id}/send-registration-link",
      post(routes::send_registration_link_to_user),
    )
    .route("/v1/user", get(routes::get_current_user))
    .route("/v1/user/groups", get(routes::get_current_user_groups))
}
