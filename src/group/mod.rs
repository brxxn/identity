use std::error::Error;

use axum::{
  Router,
  routing::{get, patch, put},
};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

use crate::{AppState, user::User};

pub mod routes;

#[derive(Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct IdentityGroup {
  pub id: i32,
  pub slug: String,
  pub name: String,
  pub description: String,
  pub is_managed: bool,
}

#[derive(Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct IdentityGroupMembership {
  pub group_id: i32,
  pub user_id: i32,
}

impl IdentityGroup {
  pub async fn fetch_all_groups(pool: &PgPool) -> Result<Vec<IdentityGroup>, Box<dyn Error>> {
    let groups = sqlx::query_as!(
      IdentityGroup,
      r#"
        SELECT 
          id, slug, name, description, is_managed
        FROM permission_groups
      "#
    )
    .fetch_all(pool)
    .await?;
    Ok(groups)
  }

  pub async fn from_group_id(pool: &PgPool, id: i32) -> Result<IdentityGroup, Box<dyn Error>> {
    let group = sqlx::query_as!(
      IdentityGroup,
      r#"
        SELECT 
          id, slug, name, description, is_managed
        FROM permission_groups WHERE id = $1
      "#,
      id
    )
    .fetch_one(pool)
    .await?;
    Ok(group)
  }

  pub async fn from_slug(pool: &PgPool, slug: String) -> Result<IdentityGroup, Box<dyn Error>> {
    let group = sqlx::query_as!(
      IdentityGroup,
      r#"
        SELECT 
          id, slug, name, description, is_managed
        FROM permission_groups WHERE slug = $1
      "#,
      slug
    )
    .fetch_one(pool)
    .await?;
    Ok(group)
  }

  pub async fn create(&mut self, pool: &PgPool) -> Result<&IdentityGroup, Box<dyn Error>> {
    let id = sqlx::query_scalar!(
      r#"
        INSERT INTO permission_groups(slug, name, description, is_managed) VALUES 
          ($1, $2, $3, $4) RETURNING id
      "#,
      self.slug,
      self.name,
      self.description,
      self.is_managed
    )
    .fetch_one(pool)
    .await?;

    self.id = id;
    Ok(self)
  }

  pub async fn update(&self, pool: &PgPool) -> Result<(), Box<dyn Error>> {
    sqlx::query!(
      r#"
        UPDATE permission_groups SET slug=$1, name=$2, description=$3, is_managed=$4
        WHERE id=$5
      "#,
      self.slug,
      self.name,
      self.description,
      self.is_managed,
      self.id
    )
    .execute(pool)
    .await?;
    Ok(())
  }

  pub async fn get_members(&self, pool: &PgPool) -> Result<Vec<User>, Box<dyn Error>> {
    let users = sqlx::query_as!(
      User,
      r#"
        SELECT u.* FROM users u
        JOIN permission_group_membership m
        ON u.id = m.user_id
        WHERE m.group_id = $1
      "#,
      self.id
    )
    .fetch_all(pool)
    .await?;
    Ok(users)
  }

  pub async fn add_member(&self, pool: &PgPool, user_id: i32) -> Result<(), Box<dyn Error>> {
    sqlx::query!(
      r#"
        INSERT INTO permission_group_membership(group_id, user_id) VALUES ($1, $2)
      "#,
      self.id,
      user_id
    )
    .execute(pool)
    .await?;
    Ok(())
  }

  /// Returns "number of rows affected" which could be either 0 (if user was not a member) or 1
  /// (if the user was a member).
  pub async fn remove_member(&self, pool: &PgPool, user_id: i32) -> Result<u64, Box<dyn Error>> {
    let result = sqlx::query!(
      r#"
        DELETE FROM permission_group_membership WHERE group_id=$1 AND user_id=$2
      "#,
      self.id,
      user_id
    )
    .execute(pool)
    .await?;
    Ok(result.rows_affected())
  }
}

pub fn router() -> Router<AppState> {
  Router::new()
    .route(
      "/v1/groups",
      get(routes::list_all_groups).post(routes::create_group),
    )
    .route("/v1/groups/{group_id}", patch(routes::update_group))
    .route(
      "/v1/groups/{group_id}/members",
      get(routes::list_all_group_members),
    )
    .route(
      "/v1/groups/{group_id}/members/{user_id}",
      put(routes::add_group_member).delete(routes::remove_group_member),
    )
}
