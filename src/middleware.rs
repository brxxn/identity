use axum::{
  extract::{Request, State},
  http::StatusCode,
  middleware::Next,
  response::Response,
};

use crate::{AppState, auth};

fn process_auth_header(state: &AppState, request: &mut Request) {
  let auth_header = match request.headers().get("authorization") {
    Some(auth) => auth.to_str().unwrap_or("None"),
    None => "None",
  };

  let split_header = auth_header.split_once(" ");
  let Some((token_type, token_value)) = split_header else {
    return;
  };

  if token_type != "Bearer" {
    return;
  }

  let Some(claims) = auth::identity::authenticate_jwt(token_value.to_string(), state) else {
    return;
  };

  request.extensions_mut().insert(claims);
}

// Generic middleware used throughout the server
pub async fn identity_auth(
  State(state): State<AppState>,
  mut request: Request,
  next: Next,
) -> Result<Response, StatusCode> {
  process_auth_header(&state, &mut request);
  Ok(next.run(request).await)
}
