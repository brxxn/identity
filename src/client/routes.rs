use axum::{
  Json,
  extract::{Path, State},
};
use serde::{Deserialize, Serialize};

use crate::{
  AppState,
  client::{IdentityClient, permissions::{GroupPermissionOverride, UserPermissionOverride}, roles::{GroupAppRoleOverride, UserAppRoleOverride}},
  response::{ApiErr, ApiResponse, EmptyResponse},
  user::AdminCtx,
};

#[derive(Deserialize)]
pub struct PartialClient {
  pub app_name: String,
  pub app_description: String,
  pub redirect_uris: Vec<String>,
  pub is_disabled: bool,
  pub default_allowed: bool,
  pub allow_implicit_flow: bool,
  pub allow_explicit_flow: bool,
}

// TODO: pagination maybe?
#[derive(Serialize)]
pub struct ListClientsResponse {
  pub clients: Vec<IdentityClient>,
}

#[derive(Serialize)]
pub struct GetClientDetailedResponse {
  pub client: IdentityClient,
  pub user_permission_overrides: Vec<UserPermissionOverride>,
  pub group_permission_overrides: Vec<GroupPermissionOverride>,
  pub user_role_overrides: Vec<UserAppRoleOverride>,
  pub group_role_overrides: Vec<GroupAppRoleOverride>,
}

#[derive(Deserialize)]
pub struct UpdateGroupPermissionOverridesRequest {
  pub group_permission_overrides: Vec<GroupPermissionOverride>
}

#[derive(Serialize)]
pub struct UpdateGroupPermissionOverridesResponse {
  pub client: IdentityClient,
  pub group_permission_overrides: Vec<GroupPermissionOverride>
}

#[derive(Deserialize)]
pub struct UpdateGroupRoleOverridesRequest {
  pub group_role_overrides: Vec<GroupAppRoleOverride>
}

#[derive(Serialize)]
pub struct UpdateGroupRoleOverridesResponse {
  pub client: IdentityClient,
  pub group_permission_overrides: Vec<GroupAppRoleOverride>
}

#[derive(Deserialize)]
pub struct UpdateUserPermissionOverrideRequest {
  pub granted: bool
}

#[derive(Deserialize)]
pub struct UpdateUserRoleOverrideRequest {
  pub granted: bool
}

#[derive(Serialize)]
pub struct UpdateClientResponse {
  pub client: IdentityClient,
}

#[derive(Serialize)]
pub struct CreateClientResponse {
  pub client: IdentityClient,
  pub client_secret: String,
}

type RotateClientSecretResponse = CreateClientResponse;

pub async fn create_client(
  State(state): State<AppState>,
  _: AdminCtx,
  Json(payload): Json<PartialClient>,
) -> ApiResponse<CreateClientResponse> {
  let mut client = IdentityClient {
    client_id: "to-be-replaced".to_string(),
    client_secret: "to-be-replaced".to_string(),
    app_name: payload.app_name,
    app_description: payload.app_description,
    redirect_uris: payload.redirect_uris,
    is_managed: false,
    is_disabled: payload.is_disabled,
    default_allowed: payload.default_allowed,
    allow_implicit_flow: payload.allow_implicit_flow,
    allow_explicit_flow: payload.allow_explicit_flow,
  };

  match client.create(&state.pool).await {
    Ok(_) => ApiResponse::Ok(CreateClientResponse {
      client_secret: client.client_secret.clone(),
      client,
    }),
    Err(_) => ApiResponse::Err(ApiErr::InternalServerError),
  }
}

pub async fn get_client_detailed(
  State(state): State<AppState>,
  _: AdminCtx,
  Path(client_id): Path<String>,
) -> ApiResponse<GetClientDetailedResponse> {
  let Ok(client) = IdentityClient::from_client_id(&state.pool, client_id.clone()).await else {
    return ApiResponse::Err(ApiErr::UnknownClient);
  };

  let Ok(user_permission_overrides) = UserPermissionOverride::get_overrides_for_client(&state.pool, client_id.clone()).await else {
    return ApiResponse::Err(ApiErr::InternalServerError);
  };

  let Ok(group_permission_overrides) = GroupPermissionOverride::fetch_group_permissions_for_client(&state.pool, client_id.clone()).await else {
    return ApiResponse::Err(ApiErr::InternalServerError);
  };

  let Ok(user_role_overrides) = UserAppRoleOverride::get_overrides_for_client(&state.pool, client_id.clone()).await else {
    return ApiResponse::Err(ApiErr::InternalServerError);
  };

  let Ok(group_role_overrides) = GroupAppRoleOverride::fetch_group_role_overrides_for_client(&state.pool, client_id.clone()).await else {
    return ApiResponse::Err(ApiErr::InternalServerError);
  };

  ApiResponse::Ok(GetClientDetailedResponse {
    client,
    user_permission_overrides,
    group_permission_overrides,
    user_role_overrides,
    group_role_overrides
  })
}

pub async fn update_client(
  State(state): State<AppState>,
  _: AdminCtx,
  Path(client_id): Path<String>,
  Json(payload): Json<PartialClient>,
) -> ApiResponse<UpdateClientResponse> {
  let Ok(mut client) = IdentityClient::from_client_id(&state.pool, client_id).await else {
    return ApiResponse::Err(ApiErr::UnknownClient);
  };

  if client.is_managed {
    return ApiResponse::Err(ApiErr::ManagedObject);
  }

  client.app_name = payload.app_name;
  client.app_description = payload.app_description;
  client.redirect_uris = payload.redirect_uris;
  client.is_disabled = payload.is_disabled;
  client.default_allowed = payload.default_allowed;
  client.allow_explicit_flow = payload.allow_explicit_flow;
  client.allow_implicit_flow = payload.allow_implicit_flow;

  match client.update(&state.pool).await {
    Ok(_) => ApiResponse::Ok(UpdateClientResponse { client }),
    Err(_) => ApiResponse::Err(ApiErr::InternalServerError),
  }
}

pub async fn rotate_client_secret(
  State(state): State<AppState>,
  _: AdminCtx,
  Path(client_id): Path<String>,
) -> ApiResponse<RotateClientSecretResponse> {
  let Ok(mut client) = IdentityClient::from_client_id(&state.pool, client_id).await else {
    return ApiResponse::Err(ApiErr::UnknownClient);
  };

  if client.is_managed {
    return ApiResponse::Err(ApiErr::ManagedObject);
  }

  match client.rotate_client_secret(&state.pool).await {
    Ok(_) => ApiResponse::Ok(RotateClientSecretResponse {
      client_secret: client.client_secret.clone(),
      client,
    }),
    Err(_) => ApiResponse::Err(ApiErr::InternalServerError),
  }
}

pub async fn update_group_permission_overrides(
  State(state): State<AppState>,
  _: AdminCtx,
  Path(client_id): Path<String>,
  Json(payload): Json<UpdateGroupPermissionOverridesRequest>
) -> ApiResponse<UpdateGroupPermissionOverridesResponse> {
  let Ok(client) = IdentityClient::from_client_id(&state.pool, client_id).await else {
    return ApiResponse::Err(ApiErr::UnknownClient);
  };

  // note: group IDs are not validated here, going to hope that's not an issue for now.
  // might resolve this at db-level by adding foreign key constraints

  let Ok(_) = GroupPermissionOverride::set_group_overrides_for_client(
    &state.pool,
    &payload.group_permission_overrides,
    client.client_id.clone()
  ).await else {
    return ApiResponse::Err(ApiErr::InternalServerError);
  };

  ApiResponse::Ok(UpdateGroupPermissionOverridesResponse { 
    client,
    group_permission_overrides: payload.group_permission_overrides
  })
}

pub async fn update_group_role_overrides(
  State(state): State<AppState>,
  _: AdminCtx,
  Path(client_id): Path<String>,
  Json(payload): Json<UpdateGroupRoleOverridesRequest>
) -> ApiResponse<UpdateGroupRoleOverridesResponse> {
  let Ok(client) = IdentityClient::from_client_id(&state.pool, client_id).await else {
    return ApiResponse::Err(ApiErr::UnknownClient);
  };

  // note: group IDs are not validated here, going to hope that's not an issue for now.
  // might resolve this at db-level by adding foreign key constraints

  let Ok(_) = GroupAppRoleOverride::upsert_group_role_overrides_for_client(
    &state.pool,
    client.client_id.clone(),
    payload.group_role_overrides.clone(),
  ).await else {
    return ApiResponse::Err(ApiErr::InternalServerError);
  };

  ApiResponse::Ok(UpdateGroupRoleOverridesResponse { 
    client,
    group_permission_overrides: payload.group_role_overrides
  })
}

pub async fn update_user_permission_override(
  State(state): State<AppState>,
  _: AdminCtx,
  Path((client_id, user_id)): Path<(String, i32)>,
  Json(payload): Json<UpdateUserPermissionOverrideRequest>
) -> ApiResponse<EmptyResponse> {
  let Ok(client) = IdentityClient::from_client_id(&state.pool, client_id).await else {
    return ApiResponse::Err(ApiErr::UnknownClient);
  };

  let permission_override = UserPermissionOverride {
    client_id: client.client_id.clone(),
    user_id,
    granted: payload.granted
  };

  match permission_override.upsert_permission_override(&state.pool).await {
    Ok(_) => ApiResponse::EmptyOk,
    Err(_) => ApiResponse::Err(ApiErr::InternalServerError)
  }
}

pub async fn update_user_role_override(
  State(state): State<AppState>,
  _: AdminCtx,
  Path((client_id, user_id, role)): Path<(String, i32, String)>,
  Json(payload): Json<UpdateUserRoleOverrideRequest>
) -> ApiResponse<EmptyResponse> {
  let Ok(client) = IdentityClient::from_client_id(&state.pool, client_id).await else {
    return ApiResponse::Err(ApiErr::UnknownClient);
  };

  let role_override = UserAppRoleOverride {
    client_id: client.client_id.clone(),
    user_id,
    granted: payload.granted,
    role
  };

  match role_override.upsert_user_role_override(&state.pool).await {
    Ok(_) => ApiResponse::EmptyOk,
    Err(_) => ApiResponse::Err(ApiErr::InternalServerError)
  }
}

pub async fn delete_user_permission_override(
  State(state): State<AppState>,
  _: AdminCtx,
  Path((client_id, user_id)): Path<(String, i32)>
) -> ApiResponse<EmptyResponse> {
  let Ok(client) = IdentityClient::from_client_id(&state.pool, client_id).await else {
    return ApiResponse::Err(ApiErr::UnknownClient);
  };

  match UserPermissionOverride::remove_permission_override(&state.pool, user_id, client.client_id).await {
    Ok(_) => ApiResponse::EmptyOk,
    Err(_) => ApiResponse::Err(ApiErr::InternalServerError)
  }
}

pub async fn delete_user_role_override(
  State(state): State<AppState>,
  _: AdminCtx,
  Path((client_id, user_id, role)): Path<(String, i32, String)>
) -> ApiResponse<EmptyResponse> {
  let Ok(client) = IdentityClient::from_client_id(&state.pool, client_id).await else {
    return ApiResponse::Err(ApiErr::UnknownClient);
  };

  let fake_override = UserAppRoleOverride {
    user_id,
    client_id: client.client_id.clone(),
    role,
    granted: false
  };

  match fake_override.remove_override(&state.pool).await {
    Ok(_) => ApiResponse::EmptyOk,
    Err(_) => ApiResponse::Err(ApiErr::InternalServerError)
  }
}

pub async fn list_all_clients(
  State(state): State<AppState>,
  _: AdminCtx,
) -> ApiResponse<ListClientsResponse> {
  match IdentityClient::fetch_all_clients(&state.pool).await {
    Ok(clients) => ApiResponse::Ok(ListClientsResponse { clients }),
    Err(_) => ApiResponse::Err(ApiErr::InternalServerError),
  }
}
