// This file just contains various utils that don't really fit anywhere else

use std::error::Error;

use base64::{Engine, prelude::BASE64_STANDARD};
use http::HeaderMap;

/// Use this to find out if a database error occurs due to a uniqueness
/// constraint failure. You can then match by the database's constraint
/// name (not the column name) to find which value has a conflict.
pub struct UniqueConstraintViolation {
  pub constraint_name: String,
}

impl UniqueConstraintViolation {
  pub fn from(err: Box<dyn Error>) -> Option<Self> {
    let database_err = err.downcast_ref::<sqlx::Error>()?.as_database_error()?;

    // 23505 is postgres uniqueness constraint
    // (duplicate key value violates unique constraint)
    if database_err.code()? != "23505" {
      return None;
    }

    Some(Self {
      constraint_name: database_err.constraint()?.to_string(),
    })
  }
}

pub fn get_basic_auth_from_header(headers: &HeaderMap) -> Option<(String, String)> {
  let auth_value = headers.get("authorization")?;
  let auth_str = auth_value.to_str().ok()?;
  let b64_data = auth_str.strip_prefix("Basic ")?;
  let auth_data = BASE64_STANDARD.decode(b64_data).ok()?;

  let utf8_auth_data = String::from_utf8(auth_data).ok()?;
  let (username, password) = utf8_auth_data.split_once(":")?;

  Some((username.to_string(), password.to_string()))
}