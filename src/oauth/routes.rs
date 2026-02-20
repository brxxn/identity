use std::collections::HashMap;

use axum::{Form, Json, extract::State, response::{IntoResponse, Response}};
use axum_auth::{AuthBearer};
use http::{HeaderMap, StatusCode};
use serde::{Deserialize, Serialize};
use webauthn_rs::prelude::Url;

use crate::{AppState, client::IdentityClient, group::IdentityGroup, oauth::{authorization::UserAppAuthorization, code::OauthCodeData, create_id_token, token::{OauthAccessTokenData, OauthRefreshTokenData}}, response::{ApiErr, ApiResponse}, user::User, util::get_basic_auth_from_header};

#[derive(Clone, Deserialize)]
pub struct OauthAuthorizeRequest {
  pub scope: String,
  pub response_type: String,
  pub client_id: String,
  pub redirect_uri: String,
  pub state: Option<String>,
  pub response_mode: Option<String>,
  pub nonce: Option<String>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct OauthTokenRequest {
  pub grant_type: String,
  pub code: Option<String>,
  pub redirect_uri: String,
  pub client_id: Option<String>,
  pub client_secret: Option<String>,

}

#[derive(Clone, Serialize, Deserialize)]
pub struct OauthTokenResponse {
  pub access_token: String,
  pub token_type: String,
  pub expires_in: u64,
  pub scope: String,
  pub refresh_token: String,
  pub id_token: String,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct OauthTokenErrorResponse {
  pub error: String,
  pub error_description: String,
}

#[derive(Serialize)]
pub struct OauthAuthorizePreviewResponse {
  pub client: IdentityClient,
}

#[derive(Serialize)]
pub struct OauthAuthorizeApproveResponse {
  pub redirect_to: String
}

fn get_oauth_error(name: &'static str, description: &'static str) -> OauthTokenErrorResponse {
  tracing::info!("encountered oauth error {} ({})", name, description);
  OauthTokenErrorResponse { error: name.to_string(), error_description: description.to_string() }
}

pub async fn validate_oauth_authorization(
  state: &AppState,
  user: &User,
  payload: &OauthAuthorizeRequest,
  client: &IdentityClient,
  groups: &Vec<IdentityGroup>
) -> Option<ApiErr> {
  if client.is_disabled {
    return Some(ApiErr::AppDisabled);
  }

  let mut valid_response_types = vec![];
  if client.allow_explicit_flow {
    valid_response_types.push("code");
  }
  if client.allow_implicit_flow {
    valid_response_types.push("token");
    valid_response_types.push("id_token");
  }
  let mut response_types: Vec<&str> = payload.response_type.split_whitespace().collect();

  if response_types.is_empty() {
    response_types.push("code");
  }

  for response_type in &response_types {
    if !valid_response_types.contains(&response_type) {
      return Some(ApiErr::Other(
        "invalid_response_type".to_string(),
        format!("Response type {} is not supported by this app. If you own this app, double check that implicit or explicit flows are enabled.", response_type)
      ));
    }
  }

  let Ok(parsed_redirect_uri) = Url::parse(&payload.redirect_uri) else {
    return Some(ApiErr::InvalidRedirectUri(payload.redirect_uri.clone()));
  };

  if parsed_redirect_uri.scheme() == "javascript" || parsed_redirect_uri.scheme() == "data" {
    return Some(ApiErr::InvalidRedirectUri(payload.redirect_uri.clone()));
  }

  if !client.redirect_uris.contains(&payload.redirect_uri) {
    return Some(ApiErr::InvalidRedirectUri(payload.redirect_uri.clone()));
  }

  let Ok(user_acl_pass) = client.is_user_allowed(&state.pool, user, groups).await else {
    return Some(ApiErr::InternalServerError);
  };

  if !user_acl_pass {
    return Some(ApiErr::OauthAclDenied(client.app_name.clone()));
  }

  if let Some(response_mode) = &payload.response_mode {
    if response_mode != "query" && response_mode != "fragment" {
      return Some(ApiErr::Other(
        "invalid_response_mode".to_string(),
        format!("Response mode {} is not supported. Valid values: query, fragment", response_mode)
      ));
    }
  }

  None
}

pub async fn oauth_authorize_preview(
  State(state): State<AppState>,
  user: User,
  Json(payload): Json<OauthAuthorizeRequest>
) -> ApiResponse<OauthAuthorizePreviewResponse> {
  let Ok(client) = IdentityClient::from_client_id(&state.pool, payload.client_id.clone()).await else {
    return ApiResponse::Err(ApiErr::UnknownClient);
  };

  let Ok(user_groups) = user.get_groups(&state.pool).await else {
    return ApiResponse::Err(ApiErr::InternalServerError);
  };
  
  if let Some(err) = validate_oauth_authorization(&state, &user, &payload, &client, &user_groups).await {
    return ApiResponse::Err(err);
  }

  ApiResponse::Ok(OauthAuthorizePreviewResponse {
    client
  })
}

pub async fn oauth_authorize_approve(
  State(state): State<AppState>,
  user: User,
  Json(payload): Json<OauthAuthorizeRequest>
) -> ApiResponse<OauthAuthorizeApproveResponse> {
  let Ok(client) = IdentityClient::from_client_id(&state.pool, payload.client_id.clone()).await else {
    return ApiResponse::Err(ApiErr::UnknownClient);
  };

  let Ok(user_groups) = user.get_groups(&state.pool).await else {
    return ApiResponse::Err(ApiErr::InternalServerError);
  };
  
  if let Some(err) = validate_oauth_authorization(&state, &user, &payload, &client, &user_groups).await {
    return ApiResponse::Err(err);
  }

  let response_types: Vec<&str> = payload.response_type.split_whitespace().collect();
  let use_fragment = match payload.response_mode {
    Some(q) => q == "fragment",
    None => response_types.contains(&"token") || response_types.contains(&"id_token")
  };

  let Ok(redirect_url) = Url::parse(&payload.redirect_uri) else {
    return ApiResponse::Err(ApiErr::InternalServerError);
  };

  let mut authorization = UserAppAuthorization {
    user_id: user.id,
    client_id: client.client_id.clone(),
    sub: "".to_string(),
    last_used: 0,
    revoked: false
  };

  if let Err(_) = authorization.authorize_for_user(&state.pool).await {
    return ApiResponse::Err(ApiErr::InternalServerError);
  }

  let mut callback_url = redirect_url.clone();
  let mut callback_params = HashMap::new();

  if response_types.contains(&"code") {
    let oauth_code_data = OauthCodeData {
      user_id: user.id,
      client_id: client.client_id.clone(),
      nonce: payload.nonce.clone(),
      redirect_uri: payload.redirect_uri.clone()
    };
    let Ok(code) = oauth_code_data.save_to_code(&state).await else {
      return ApiResponse::Err(ApiErr::InternalServerError);
    };
    callback_params.insert("code", code);
  }

  if response_types.contains(&"token") {
    let oauth_access_token_data = OauthAccessTokenData {
      user_id: user.id,
      client_id: client.client_id.clone(),
      nonce: payload.nonce.clone()
    };
    let Ok(token) = oauth_access_token_data.save_to_token(&state).await else {
      return ApiResponse::Err(ApiErr::InternalServerError);
    };
    callback_params.insert("access_token", token);
    callback_params.insert("token_type", "bearer".to_string());
    callback_params.insert("expires_in", "3600".to_string());
  }

  if response_types.contains(&"id_token") {
    let Ok(id_token) = create_id_token(&state, &user, &client, user_groups, payload.nonce.clone(), &authorization).await else {
      return ApiResponse::Err(ApiErr::InternalServerError);
    };
    callback_params.insert("id_token", id_token);
  }

  if let Some(state) = payload.state {
    callback_params.insert("state", state);
  }

  let Ok(parameter_string) = serde_urlencoded::to_string(callback_params) else {
    return ApiResponse::Err(ApiErr::InternalServerError);
  };

  if use_fragment {
    callback_url.set_fragment(Some(parameter_string.as_str()));
    callback_url.set_query(None);
  } else {
    callback_url.set_query(Some(parameter_string.as_str()));
    callback_url.set_fragment(None);
  }

  ApiResponse::Ok(OauthAuthorizeApproveResponse {
    redirect_to: callback_url.to_string()
  })
}

pub async fn oauth_token(
  State(state): State<AppState>,
  headers: HeaderMap,
  Form(payload): Form<OauthTokenRequest>
) -> Response {
  let (client_id, client_secret) = match get_basic_auth_from_header(&headers) {
    Some((client_id, client_secret)) => (client_id, client_secret),
    None => {
      match payload.client_id {
        Some(client_id) => match payload.client_secret {
          Some(client_secret) => {
            (client_id, client_secret)
          },
          None => {
            return (StatusCode::BAD_REQUEST, Json(get_oauth_error(
              "invalid_request",
              "client_secret is required (PKCE authentication is not yet implemented)."
            ))).into_response();
          }
        },
        None => {
          return (StatusCode::BAD_REQUEST, Json(get_oauth_error(
            "invalid_request",
            "client_id must be provided"
          ))).into_response();
        }
      }
    }
  };

  let Ok(client) = IdentityClient::from_client_id(&state.pool, client_id).await else {
    return (StatusCode::BAD_REQUEST, Json(get_oauth_error(
      "invalid_client",
      "Client could not be found or has invalid secret"
    ))).into_response();
  };

  // maybe do some fancy xor constant time bullshit in the future
  if client.client_secret != client_secret || client.is_disabled {
    return (StatusCode::BAD_REQUEST, Json(get_oauth_error(
      "invalid_client",
      "Client could not be found or has invalid secret"
    ))).into_response();
  }

  match payload.grant_type.as_str() {
    "authorization_code" => {
      let Some(code) = payload.code else {
        return (StatusCode::BAD_REQUEST, Json(get_oauth_error(
          "invalid_request",
          "Code parameter required when using authorization_code"
        ))).into_response();
      };
      
      let Ok(code_opt) = OauthCodeData::from_code(&state, code).await else {
        return (StatusCode::INTERNAL_SERVER_ERROR, Json(get_oauth_error(
          "internal_server_error",
          "Something went wrong!"
        ))).into_response();
      };

      let code_not_valid = (StatusCode::BAD_REQUEST, Json(get_oauth_error(
        "invalid_grant",
        "Code not valid"
      ))).into_response();

      let Some(code_data) = code_opt else {
        return code_not_valid;
      };

      if code_data.redirect_uri != payload.redirect_uri || code_data.client_id != client.client_id {
        return code_not_valid;
      }

      let Ok(user) = User::from_user_id(&state.pool, code_data.user_id).await else {
        return code_not_valid;
      };

      let Ok(user_app_auth) = UserAppAuthorization::get_authorization(&state.pool, user.id, client.client_id.clone()).await else {
        return code_not_valid;
      };
      
      if user_app_auth.revoked {
        return code_not_valid;
      }

      let Ok(groups) = user.get_groups(&state.pool).await else {
        return (StatusCode::INTERNAL_SERVER_ERROR, Json(get_oauth_error(
          "internal_server_error",
          "Something went wrong!"
        ))).into_response();
      };

      let Ok(user_permission) = client.is_user_allowed(&state.pool, &user, &groups).await else {
        return (StatusCode::INTERNAL_SERVER_ERROR, Json(get_oauth_error(
          "internal_server_error",
          "Something went wrong!"
        ))).into_response();
      };

      if !user_permission {
        return code_not_valid;
      }

      let Ok(id_token) = create_id_token(&state, &user, &client, groups, code_data.nonce.clone(), &user_app_auth).await else {
        return (StatusCode::INTERNAL_SERVER_ERROR, Json(get_oauth_error(
          "internal_server_error",
          "Something went wrong!"
        ))).into_response();
      };

      let access_token_data = OauthAccessTokenData {
        user_id: user.id,
        client_id: client.client_id.clone(),
        nonce: code_data.nonce.clone()
      };

      let Ok(access_token) = access_token_data.save_to_token(&state).await else {
        return (StatusCode::INTERNAL_SERVER_ERROR, Json(get_oauth_error(
          "internal_server_error",
          "Something went wrong!"
        ))).into_response();
      };

      let refresh_token_data = OauthRefreshTokenData {
        user_id: user.id,
        client_id: client.client_id.clone(),
        nonce: code_data.nonce
      };

      let Ok(refresh_token) = refresh_token_data.save_to_token(&state).await else {
        return (StatusCode::INTERNAL_SERVER_ERROR, Json(get_oauth_error(
          "internal_server_error",
          "Something went wrong!"
        ))).into_response();
      };

      return (StatusCode::OK, Json(OauthTokenResponse {
        access_token,
        token_type: "Bearer".to_string(),
        expires_in: 3600,
        scope: "openid profile email".to_string(),
        refresh_token,
        id_token,
      })).into_response();
    },
    _ => (StatusCode::BAD_REQUEST, Json(get_oauth_error(
      "unsupported_grant_type",
      "Grant type not supported by server!"
    ))).into_response()
  }
}

pub async fn oauth_userinfo(
  State(state): State<AppState>,
  AuthBearer(access_token): AuthBearer
) -> Response {
  let mut invalid_token_headers = HeaderMap::new();
  invalid_token_headers.insert("www-authenticate", "Bearer error=\"invalid_token\"".parse().unwrap());

  let Ok(access_token_opt) = OauthAccessTokenData::from_token(&state, access_token).await else {
    return StatusCode::INTERNAL_SERVER_ERROR.into_response();
  };

  let Some(access_token_data) = access_token_opt else {
    return (StatusCode::UNAUTHORIZED, invalid_token_headers).into_response();
  };

  let Ok(user_app_auth) = UserAppAuthorization::get_authorization(&state.pool, access_token_data.user_id, access_token_data.client_id.clone()).await else {
    return (StatusCode::UNAUTHORIZED, invalid_token_headers).into_response();
  };

  let Ok(user) = User::from_user_id(&state.pool, access_token_data.user_id).await else {
    return (StatusCode::UNAUTHORIZED, invalid_token_headers).into_response();
  };

  if user.is_suspended {
    return (StatusCode::UNAUTHORIZED, invalid_token_headers).into_response();
  }

  let Ok(client) = IdentityClient::from_client_id(&state.pool, access_token_data.client_id).await else {
    return (StatusCode::UNAUTHORIZED, invalid_token_headers).into_response();
  };

  if client.is_disabled {
    return (StatusCode::UNAUTHORIZED, invalid_token_headers).into_response();
  }

  let Ok(groups) = user.get_groups(&state.pool).await else {
    return (StatusCode::UNAUTHORIZED, invalid_token_headers).into_response();
  };

  let Ok(user_permission) = client.is_user_allowed(&state.pool, &user, &groups).await else {
    return StatusCode::INTERNAL_SERVER_ERROR.into_response();
  };

  if !user_permission {
    return (StatusCode::UNAUTHORIZED, invalid_token_headers).into_response();
  }

  let Ok(id_token) = create_id_token(&state, &user, &client, groups, access_token_data.nonce.clone(), &user_app_auth).await else {
    return (StatusCode::UNAUTHORIZED, invalid_token_headers).into_response();
  };

  let mut ok_resp_headers = HeaderMap::new();
  ok_resp_headers.insert("content-type", "application/jwt".parse().unwrap());

  (ok_resp_headers, id_token).into_response()
}