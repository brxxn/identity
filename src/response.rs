use axum::{Json, http::StatusCode, response::IntoResponse};
use serde::Serialize;

#[derive(Serialize)]
struct SuccessBody<T>
where
  T: Serialize,
{
  pub data: T,
}

#[derive(Serialize)]
struct ErrorBody<T>
where
  T: Serialize,
{
  pub error: T,
}

// For generic/common errors
#[derive(Serialize)]
struct ErrorMessage {
  pub code: String,
  pub message: String,
}

#[derive(Serialize)]
pub struct EmptyResponse {}

pub enum ApiErr {
  InvalidChallenge,
  ExpiredRegistration,
  InvalidCredential,
  UserDeleted,
  UserSuspended,
  InternalServerError,
  SessionExpired,
  LoginRequired,
  AdminRequired,
  UnknownClient,
  UnknownGroup,
  UnknownUser,
  GroupSlugExists,
  UsernameExists,
  EmailExists,
  AppDisabled,
  ManagedObject,
  GenericError,
  OauthAclDenied(String),
  InvalidRedirectUri(String),
  Other(String, String),
}

pub enum ApiResponse<T>
where
  T: Serialize,
{
  Ok(T),
  Err(ApiErr),
  EmptyOk,
}

fn error_msg(code: &'static str, message: &'static str) -> ErrorMessage {
  ErrorMessage {
    code: code.to_string(),
    message: message.to_string(),
  }
}

impl ApiErr {
  fn serialize(self) -> ErrorMessage {
    match self {
      ApiErr::InvalidChallenge => error_msg(
        "invalid_challenge",
        "You took too long to complete the authentication flow, please try again.",
      ),
      ApiErr::ExpiredRegistration => error_msg(
        "expired_registration",
        "This registration link appears to have expired. Please ask an administrator for another link.",
      ),
      ApiErr::InvalidCredential => error_msg(
        "invalid_credential",
        "This passkey is not valid or has been removed from the account you are trying to sign into.",
      ),
      ApiErr::UserDeleted => error_msg(
        "user_deleted",
        "It looks like this account has been deleted or no longer exists.",
      ),
      ApiErr::UserSuspended => error_msg(
        "user_suspended",
        "This action is currently unavailable because your account is suspended.",
      ),
      ApiErr::InternalServerError => error_msg(
        "internal_server_error",
        "An unknown exception occurred, please try again later.",
      ),
      ApiErr::SessionExpired => error_msg(
        "session_expired",
        "Your session is no longer valid, and you will need to sign in again.",
      ),
      ApiErr::UnknownClient => error_msg(
        "unknown_client",
        "Sorry, but we couldn't find this app. This can occur if the app was deleted or the client_id parameter is incorrect.",
      ),
      ApiErr::UnknownGroup => error_msg(
        "unknown_group",
        "Sorry, but this group doesn't exist or has been deleted.",
      ),
      ApiErr::UnknownUser => error_msg(
        "unknown_user",
        "Sorry, but this user doesn't exist or has been deleted.",
      ),
      ApiErr::GroupSlugExists => error_msg(
        "group_slug_exists",
        "The group slug you provided is already in use by another group.",
      ),
      ApiErr::UsernameExists => error_msg(
        "username_exists",
        "This username is already in use by another user.",
      ),
      ApiErr::EmailExists => error_msg(
        "username_exists",
        "This email is already in use by another user.",
      ),
      ApiErr::AppDisabled => {
        error_msg("app_disabled", "Sorry, but this app is currently disabled.")
      }
      ApiErr::ManagedObject => error_msg(
        "managed_object",
        "You can't do that because doing so may cause issues with the identity server.",
      ),
      ApiErr::LoginRequired => {
        error_msg("login_required", "You must login to perform this action.")
      }
      ApiErr::AdminRequired => error_msg("admin_required", "You don't have permission to do that."),
      ApiErr::OauthAclDenied(name) => ErrorMessage {
        code: "oauth_acl_denied".to_string(),
        message: format!(
          "You cannot access {} because you are failing the ACL checks to use this service. An administrator may need to add you to an identity group or grant you permission to use this app.",
          name
        ),
      },
      ApiErr::InvalidRedirectUri(redirect_uri) => ErrorMessage {
        code: "invalid_redirect_uri".to_string(),
        message: format!(
          "The redirect_uri {} is not valid. Check that it exactly matches one of the added URIs for this app and is a compliant OAuth redirect URI.",
          redirect_uri
        ),
      },
      ApiErr::Other(code, message) => ErrorMessage { code, message },
      _ => error_msg("unknown_error", "An error occurred."),
    }
  }

  fn status(&self) -> StatusCode {
    match self {
      ApiErr::InvalidChallenge => StatusCode::FORBIDDEN,
      ApiErr::ExpiredRegistration => StatusCode::FORBIDDEN,
      ApiErr::UserSuspended => StatusCode::FORBIDDEN,
      ApiErr::InternalServerError => StatusCode::INTERNAL_SERVER_ERROR,
      ApiErr::LoginRequired => StatusCode::UNAUTHORIZED,
      ApiErr::AdminRequired => StatusCode::FORBIDDEN,
      ApiErr::Other(_, _) => StatusCode::BAD_REQUEST,
      _ => StatusCode::BAD_REQUEST,
    }
  }
}

impl<T> IntoResponse for ApiResponse<T>
where
  T: Serialize,
{
  fn into_response(self) -> axum::response::Response {
    match self {
      ApiResponse::Ok(data) => Json(SuccessBody { data }).into_response(),
      ApiResponse::Err(err) => (
        err.status(),
        Json(ErrorBody {
          error: err.serialize(),
        }),
      )
        .into_response(),
      ApiResponse::EmptyOk => StatusCode::NO_CONTENT.into_response(),
    }
  }
}
