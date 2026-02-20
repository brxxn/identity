use std::error::Error;

use axum::{
  Router,
  routing::{get, patch, post},
};
use rand::distributions::{Alphanumeric, DistString};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

use crate::{
  AppState,
  client::{
    permissions::{GroupPermissionOverride, UserPermissionOverride},
    roles::{GroupAppRoleOverride, UserAppRoleOverride},
  },
  group::IdentityGroup,
  user::User,
};

pub mod permissions;
pub mod roles;
pub mod routes;

#[derive(Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct IdentityClient {
  pub client_id: String,
  #[serde(skip)]
  pub client_secret: String,
  pub app_name: String,
  pub app_description: String,
  pub redirect_uris: Vec<String>,
  pub is_managed: bool,
  pub is_disabled: bool,
  pub default_allowed: bool,
  pub allow_explicit_flow: bool,
  pub allow_implicit_flow: bool,
}

impl IdentityClient {
  pub async fn fetch_all_clients(pool: &PgPool) -> Result<Vec<IdentityClient>, Box<dyn Error>> {
    let clients = sqlx::query_as!(
      IdentityClient,
      r#"
        SELECT 
          client_id, client_secret, app_name, app_description, redirect_uris, is_managed, is_disabled, default_allowed, allow_explicit_flow, allow_implicit_flow
        FROM clients
      "#
    ).fetch_all(pool).await?;
    Ok(clients)
  }

  pub async fn from_client_id(
    pool: &PgPool,
    client_id: String,
  ) -> Result<IdentityClient, Box<dyn Error>> {
    let client = sqlx::query_as!(
      IdentityClient,
      r#"
        SELECT 
          client_id, client_secret, app_name, app_description, redirect_uris, is_managed, is_disabled, default_allowed, allow_explicit_flow, allow_implicit_flow
        FROM clients WHERE client_id = $1
      "#,
      client_id
    ).fetch_one(pool).await?;
    Ok(client)
  }

  pub async fn create(&mut self, pool: &PgPool) -> Result<&IdentityClient, Box<dyn Error>> {
    let mut client_id_generator = snowflaked::Generator::new(0);
    let client_id = client_id_generator.generate::<i64>().to_string();
    let client_secret = Alphanumeric.sample_string(&mut rand::thread_rng(), 64);

    self.client_id = client_id;
    self.client_secret = client_secret;

    sqlx::query!(
      r#"
        INSERT INTO clients(client_id, client_secret, app_name, app_description, redirect_uris, is_managed, is_disabled, default_allowed, allow_explicit_flow, allow_implicit_flow) VALUES 
          ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
      "#,
      self.client_id, self.client_secret, self.app_name, self.app_description, self.redirect_uris.as_slice(), self.is_managed, self.is_disabled, self.default_allowed, self.allow_explicit_flow, self.allow_implicit_flow
    ).execute(pool).await?;

    Ok(self)
  }

  pub async fn update(&self, pool: &PgPool) -> Result<(), Box<dyn Error>> {
    sqlx::query!(
      r#"
        UPDATE clients SET client_secret=$1, app_name=$2, app_description=$3, redirect_uris=$4, is_managed=$5, is_disabled=$6, default_allowed=$7, allow_implicit_flow=$8, allow_explicit_flow=$9
        WHERE client_id=$10
      "#,
      self.client_secret, self.app_name, self.app_description, self.redirect_uris.as_slice(), self.is_managed, self.is_disabled, self.default_allowed, self.allow_implicit_flow, self.allow_explicit_flow, self.client_id
    ).execute(pool).await?;
    Ok(())
  }

  pub async fn rotate_client_secret(&mut self, pool: &PgPool) -> Result<(), Box<dyn Error>> {
    let client_secret = Alphanumeric.sample_string(&mut rand::thread_rng(), 64);
    self.client_secret = client_secret;

    return self.update(&pool).await;
  }

  pub async fn is_user_allowed(
    &self,
    pool: &PgPool,
    user: &User,
    groups: &Vec<IdentityGroup>,
  ) -> Result<bool, Box<dyn Error>> {
    let mut allow = self.default_allowed;

    // if there's a user override, we should apply it immediately and short-circuit other checks.
    let user_override_opt = UserPermissionOverride::fetch_user_permissions_for_client(
      pool,
      user.id,
      self.client_id.clone(),
    )
    .await?;

    if let Some(user_override) = user_override_opt {
      return Ok(user_override.granted);
    }

    let group_ids = groups.iter().map(|x| x.id).collect::<Vec<i32>>();

    let mut group_permissions =
      GroupPermissionOverride::fetch_group_permissions_for_client(pool, self.client_id.clone())
        .await?;
    group_permissions.sort_by_key(|x| x.override_priority);

    for permission in &group_permissions {
      if !group_ids.contains(&permission.group_id) {
        continue;
      }

      allow = permission.granted;
    }

    Ok(allow)
  }

  pub async fn get_user_roles(
    &self,
    pool: &PgPool,
    user: &User,
    groups: &Vec<IdentityGroup>,
  ) -> Result<Vec<String>, Box<dyn Error>> {
    let mut roles = Vec::new();

    // we have to do group first to get the group overrides and then apply the user on top
    let group_ids = groups.iter().map(|x| x.id).collect::<Vec<i32>>();

    let mut group_overrides =
      GroupAppRoleOverride::fetch_group_role_overrides_for_client(pool, self.client_id.clone())
        .await?;
    group_overrides.sort_by_key(|x| x.override_priority);

    for role_override in &group_overrides {
      if !group_ids.contains(&role_override.group_id) {
        continue;
      }

      if *&role_override.granted && !roles.contains(&role_override.role) {
        roles.push(role_override.role.clone());
      } else if !*&role_override.granted && roles.contains(&role_override.role) {
        roles.retain(|x| *x != role_override.role)
      }
    }

    // if there's a user override, we should apply it immediately and short-circuit other checks.
    let user_overrides = UserAppRoleOverride::fetch_user_role_overrides_for_client(
      pool,
      user.id,
      self.client_id.clone(),
    )
    .await?;

    for role_override in &user_overrides {
      if *&role_override.granted && !roles.contains(&role_override.role) {
        roles.push(role_override.role.clone());
      } else if !*&role_override.granted && roles.contains(&role_override.role) {
        roles.retain(|x| *x != role_override.role)
      }
    }

    Ok(roles)
  }
}

pub fn router() -> Router<AppState> {
  Router::new()
    .route(
      "/v1/clients",
      get(routes::list_all_clients).post(routes::create_client),
    )
    .route(
      "/v1/clients/{client_id}",
      get(routes::get_client_detailed).patch(routes::update_client),
    )
    .route(
      "/v1/clients/{client_id}/rotate-secret",
      post(routes::rotate_client_secret),
    )
    .route(
      "/v1/clients/{client_id}/group-overrides/permissions",
      patch(routes::update_group_permission_overrides),
    )
    .route(
      "/v1/clients/{client_id}/group-overrides/roles",
      patch(routes::update_group_role_overrides),
    )
    .route(
      "/v1/clients/{client_id}/user-overrides/{user_id}/permission",
      patch(routes::update_user_permission_override)
        .delete(routes::delete_user_permission_override),
    )
    .route(
      "/v1/clients/{client_id}/user-overrides/{user_id}/roles/{role}",
      patch(routes::update_user_role_override).delete(routes::delete_user_role_override),
    )
}
