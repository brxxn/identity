use axum::{
  Json,
  extract::{Path, State},
};
use serde::{Deserialize, Serialize};

use crate::{
  AppState,
  group::IdentityGroup,
  response::{ApiErr, ApiResponse},
  user::{AdminCtx, User},
  util::UniqueConstraintViolation,
};

#[derive(Deserialize)]
pub struct PartialGroup {
  pub slug: String,
  pub name: String,
  pub description: String,
}

// TODO: pagination maybe?
#[derive(Serialize)]
pub struct ListGroupsResponse {
  pub groups: Vec<IdentityGroup>,
}

#[derive(Serialize)]
pub struct ListGroupMembersResponse {
  pub group: IdentityGroup,
  pub members: Vec<User>,
}

#[derive(Serialize)]
pub struct CreateGroupResponse {
  pub group: IdentityGroup,
}

#[derive(Serialize)]
pub struct AddGroupMemberResponse {
  pub group: IdentityGroup,
  pub targeted_user: User,
  pub members: Vec<User>,
}

type UpdateGroupResponse = CreateGroupResponse;
type RemoveGroupMemberResponse = AddGroupMemberResponse;

pub async fn create_group(
  State(state): State<AppState>,
  _: AdminCtx,
  Json(payload): Json<PartialGroup>,
) -> ApiResponse<CreateGroupResponse> {
  let mut group = IdentityGroup {
    id: 0,
    slug: payload.slug,
    name: payload.name,
    description: payload.description,
    is_managed: false,
  };

  match group.create(&state.pool).await {
    Ok(_) => ApiResponse::Ok(CreateGroupResponse { group }),
    Err(err) => match UniqueConstraintViolation::from(err) {
      Some(violation) => match violation.constraint_name.as_str() {
        "permission_groups_slug_key" => ApiResponse::Err(ApiErr::GroupSlugExists),
        _ => ApiResponse::Err(ApiErr::InternalServerError),
      },
      None => ApiResponse::Err(ApiErr::InternalServerError),
    },
  }
}

pub async fn update_group(
  State(state): State<AppState>,
  _: AdminCtx,
  Path(group_id): Path<i32>,
  Json(payload): Json<PartialGroup>,
) -> ApiResponse<UpdateGroupResponse> {
  let Ok(mut group) = IdentityGroup::from_group_id(&state.pool, group_id).await else {
    return ApiResponse::Err(ApiErr::UnknownGroup);
  };

  if group.is_managed {
    return ApiResponse::Err(ApiErr::ManagedObject);
  }

  group.name = payload.name;
  group.description = payload.description;
  group.slug = payload.slug;

  match group.update(&state.pool).await {
    Ok(_) => ApiResponse::Ok(UpdateGroupResponse { group }),
    Err(err) => match UniqueConstraintViolation::from(err) {
      Some(violation) => match violation.constraint_name.as_str() {
        "permission_groups_slug_key" => ApiResponse::Err(ApiErr::GroupSlugExists),
        _ => ApiResponse::Err(ApiErr::InternalServerError),
      },
      None => ApiResponse::Err(ApiErr::InternalServerError),
    },
  }
}

pub async fn list_all_groups(
  State(state): State<AppState>,
  _: AdminCtx,
) -> ApiResponse<ListGroupsResponse> {
  match IdentityGroup::fetch_all_groups(&state.pool).await {
    Ok(groups) => ApiResponse::Ok(ListGroupsResponse { groups }),
    Err(_) => ApiResponse::Err(ApiErr::InternalServerError),
  }
}

pub async fn list_all_group_members(
  State(state): State<AppState>,
  _: AdminCtx,
  Path(group_id): Path<i32>,
) -> ApiResponse<ListGroupMembersResponse> {
  let Ok(group) = IdentityGroup::from_group_id(&state.pool, group_id).await else {
    return ApiResponse::Err(ApiErr::UnknownGroup);
  };

  match group.get_members(&state.pool).await {
    Ok(members) => ApiResponse::Ok(ListGroupMembersResponse { group, members }),
    Err(_) => ApiResponse::Err(ApiErr::InternalServerError),
  }
}

pub async fn add_group_member(
  State(state): State<AppState>,
  _: AdminCtx,
  Path((group_id, user_id)): Path<(i32, i32)>,
) -> ApiResponse<AddGroupMemberResponse> {
  let Ok(group) = IdentityGroup::from_group_id(&state.pool, group_id).await else {
    return ApiResponse::Err(ApiErr::UnknownGroup);
  };

  if group.is_managed {
    return ApiResponse::Err(ApiErr::ManagedObject);
  }

  let Ok(user) = User::from_user_id(&state.pool, user_id).await else {
    return ApiResponse::Err(ApiErr::UnknownUser);
  };

  // If a conflicting primary key exists, we want to return a fake success because the action has
  // already happened and this will allow outdated clients to reflect that change without weird errors
  // or having to refresh.
  if let Err(err) = group.add_member(&state.pool, user.id).await {
    if let Some(violation) = UniqueConstraintViolation::from(err) {
      if violation.constraint_name != "permission_group_membership_pkey" {
        return ApiResponse::Err(ApiErr::InternalServerError);
      }
    } else {
      return ApiResponse::Err(ApiErr::InternalServerError);
    }
  }

  match group.get_members(&state.pool).await {
    Ok(members) => ApiResponse::Ok(AddGroupMemberResponse {
      group,
      members,
      targeted_user: user,
    }),
    Err(_) => ApiResponse::Err(ApiErr::InternalServerError),
  }
}

pub async fn remove_group_member(
  State(state): State<AppState>,
  _: AdminCtx,
  Path((group_id, user_id)): Path<(i32, i32)>,
) -> ApiResponse<RemoveGroupMemberResponse> {
  let Ok(group) = IdentityGroup::from_group_id(&state.pool, group_id).await else {
    return ApiResponse::Err(ApiErr::UnknownGroup);
  };

  if group.is_managed {
    return ApiResponse::Err(ApiErr::ManagedObject);
  }

  let Ok(user) = User::from_user_id(&state.pool, user_id).await else {
    return ApiResponse::Err(ApiErr::UnknownUser);
  };

  let Ok(rows) = group.remove_member(&state.pool, user.id).await else {
    return ApiResponse::Err(ApiErr::InternalServerError);
  };

  // I think attempting to remove a user that doesn't exist warrants more of an
  // error, since it is possible the admin is intending to remove a real user rather
  // than experiencing a race condition.
  if rows == 0 {
    return ApiResponse::Err(ApiErr::Other(
      "user_not_in_group".to_string(), 
      "The targeted user is not in the group you are trying to remove them from. This may mean they have already been removed.".to_string()
    ));
  }

  match group.get_members(&state.pool).await {
    Ok(members) => ApiResponse::Ok(AddGroupMemberResponse {
      group,
      members,
      targeted_user: user,
    }),
    Err(_) => ApiResponse::Err(ApiErr::InternalServerError),
  }
}
