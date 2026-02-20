use axum::{
  Json,
  extract::{Path, State},
};
use serde::{Deserialize, Serialize};
use sqlx::types::Uuid;

use crate::{
  AppState,
  group::IdentityGroup,
  response::{ApiErr, ApiResponse, EmptyResponse},
  user::{AdminCtx, User},
  util::UniqueConstraintViolation,
};

#[derive(Deserialize)]
pub struct PartialUser {
  pub email: String,
  pub username: String,
  pub name: String,
  pub is_suspended: bool,
  pub is_admin: bool,
}

// TODO: pagination maybe?
#[derive(Serialize)]
pub struct ListUsersResponse {
  pub users: Vec<User>,
}

#[derive(Serialize)]
pub struct GetUserResponse {
  pub user: User,
  pub groups: Vec<IdentityGroup>,
}

#[derive(Serialize)]
pub struct UpdateUserResponse {
  pub user: User,
}

type CreateUserResponse = UpdateUserResponse;

// ---- Admin Routes ----

pub async fn list_users(
  State(state): State<AppState>,
  _: AdminCtx,
) -> ApiResponse<ListUsersResponse> {
  match User::list_all_users(&state.pool).await {
    Ok(users) => ApiResponse::Ok(ListUsersResponse { users }),
    Err(_) => ApiResponse::Err(ApiErr::InternalServerError),
  }
}

pub async fn get_user_by_id(
  State(state): State<AppState>,
  _: AdminCtx,
  Path(user_id): Path<i32>,
) -> ApiResponse<GetUserResponse> {
  let Ok(user) = User::from_user_id(&state.pool, user_id).await else {
    return ApiResponse::Err(ApiErr::UnknownUser);
  };

  let Ok(groups) = user.get_groups(&state.pool).await else {
    return ApiResponse::Err(ApiErr::InternalServerError);
  };

  ApiResponse::Ok(GetUserResponse { user, groups })
}

pub async fn update_user(
  State(state): State<AppState>,
  _: AdminCtx,
  Path(user_id): Path<i32>,
  Json(payload): Json<PartialUser>,
) -> ApiResponse<UpdateUserResponse> {
  let Ok(mut user) = User::from_user_id(&state.pool, user_id).await else {
    return ApiResponse::Err(ApiErr::UnknownUser);
  };

  user.name = payload.name;
  user.email = payload.email;
  user.username = payload.username;
  user.is_suspended = payload.is_suspended;
  user.is_admin = payload.is_admin;

  match user.update(&state.pool).await {
    Ok(_) => ApiResponse::Ok(UpdateUserResponse { user }),
    Err(err) => match UniqueConstraintViolation::from(err) {
      Some(violation) => match violation.constraint_name.as_str() {
        "users_username_key" => ApiResponse::Err(ApiErr::UsernameExists),
        "users_email_key" => ApiResponse::Err(ApiErr::EmailExists),
        _ => ApiResponse::Err(ApiErr::InternalServerError),
      },
      None => ApiResponse::Err(ApiErr::InternalServerError),
    },
  }
}

pub async fn create_user(
  State(state): State<AppState>,
  _: AdminCtx,
  Json(payload): Json<PartialUser>,
) -> ApiResponse<UpdateUserResponse> {
  let mut user = User {
    id: 0,
    email: payload.email,
    username: payload.username,
    name: payload.name,
    is_suspended: payload.is_suspended,
    is_admin: payload.is_admin,
    credential_uuid: Uuid::new_v4(),
  };

  // TODO: consider automatically sending out registration email?

  match user.create(&state.pool).await {
    Ok(_) => ApiResponse::Ok(CreateUserResponse { user }),
    Err(err) => match UniqueConstraintViolation::from(err) {
      Some(violation) => match violation.constraint_name.as_str() {
        "users_username_key" => ApiResponse::Err(ApiErr::UsernameExists),
        "users_email_key" => ApiResponse::Err(ApiErr::EmailExists),
        _ => ApiResponse::Err(ApiErr::InternalServerError),
      },
      None => ApiResponse::Err(ApiErr::InternalServerError),
    },
  }
}

pub async fn send_registration_link_to_user(
  State(state): State<AppState>,
  _: AdminCtx,
  Path(user_id): Path<i32>,
) -> ApiResponse<EmptyResponse> {
  let Ok(user) = User::from_user_id(&state.pool, user_id).await else {
    return ApiResponse::Err(ApiErr::UnknownUser);
  };

  match user.send_registration_mail(&state).await {
    Ok(_) => ApiResponse::EmptyOk,
    Err(_) => ApiResponse::Err(ApiErr::InternalServerError),
  }
}

// ---- User Routes ----

pub async fn get_current_user(current_user: User) -> ApiResponse<User> {
  ApiResponse::Ok(current_user)
}

pub async fn get_current_user_groups(
  State(state): State<AppState>,
  current_user: User,
) -> ApiResponse<GetUserResponse> {
  let Ok(groups) = current_user.get_groups(&state.pool).await else {
    return ApiResponse::Err(ApiErr::InternalServerError);
  };

  ApiResponse::Ok(GetUserResponse {
    user: current_user,
    groups,
  })
}
