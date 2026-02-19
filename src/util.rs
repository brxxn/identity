// This file just contains various utils that don't really fit anywhere else

use std::error::Error;

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
