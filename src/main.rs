use std::{collections::HashMap, env, error::Error, path::Path, sync::Arc};

use axum::Router;
use http::Method;
use lettre::{AsyncSmtpTransport, transport::smtp::authentication::Credentials};
use redis::aio::MultiplexedConnection;
use rsa::RsaPrivateKey;
use sqlx::postgres::PgPoolOptions;
use webauthn_rs::{Webauthn, WebauthnBuilder, prelude::Url};

use crate::{cli::{handle_email_cli, handle_setup_cli}, keys::load_keys};

pub mod auth;
pub mod client;
pub mod cli;
pub mod frontend;
pub mod group;
pub mod keys;
pub mod middleware;
pub mod oauth;
pub mod response;
pub mod smtp;
pub mod user;
pub mod util;

#[derive(Clone)]
pub struct AppPrivateKeys {
  pub passkey_registration_key: String,
  pub passkey_authentication_key: String,
  pub oidc_jwt_keys: HashMap<u64, RsaPrivateKey>,
  pub identity_access_jwt_key: String,
  pub identity_refresh_jwt_key: String,
  pub registration_jwt_key: String,
}

#[derive(Clone)]
pub struct AppMailer {
  pub transport: Arc<AsyncSmtpTransport<lettre::Tokio1Executor>>,
  pub sender: String,
}

#[derive(Clone)]
pub struct AppState {
  pub pool: sqlx::PgPool,
  pub private_keys: AppPrivateKeys,
  pub webauthn: Webauthn,
  pub mailer: Option<AppMailer>,
  pub oidc_issuer_uri: String,
  pub redis_connection: MultiplexedConnection,
}

fn extract_from_env(key: &'static str, default: &'static str) -> String {
  match env::var(key) {
    Ok(val) => val,
    Err(_) => {
      tracing::warn!(
        "Environment variable {} not found, falling back to default \"{}\"",
        key,
        default
      );
      default.to_string()
    }
  }
}

async fn shutdown_signal() {
  let ctrl_c = async {
    tokio::signal::ctrl_c()
      .await
      .expect("failed to install keyboard interrupt listener");
  };

  // should be true since we will always be in docker
  #[cfg(unix)]
  let terminate = async {
    tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
      .expect("failed to install unix terminate signal handler!")
      .recv()
      .await;
  };

  #[cfg(not(unix))]
  let terminate = std::future::pending::<()>();

  tokio::select! {
    _ = ctrl_c => {},
    _ = terminate => {},
  }

  tracing::info!("Received shutdown signal, terminating...");
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
  // dotenvy::dotenv()?;
  tracing_subscriber::fmt::init();

  let postgres_user = extract_from_env("POSTGRES_USER", "postgres");
  let postgres_password = extract_from_env("POSTGRES_PASSWORD", "password");
  let postgres_host = extract_from_env("POSTGRES_HOST", "postgres");
  let postgres_database = extract_from_env("POSTGRES_DB", "identity");

  let webauthn_rp_id = extract_from_env("WEBAUTHN_RP_ID", "identity.example.com");
  let webauthn_rp_origin = extract_from_env("WEBAUTHN_RP_ORIGIN", "https://identity.example.com");

  let key_dir = extract_from_env("KEYS_DIR", "/keys");
  let frontend_str = extract_from_env("FRONTEND_DIR", "/frontend/dist");

  let smtp_enabled = extract_from_env("SMTP_ENABLED", "0");
  let smtp_hostname = extract_from_env("SMTP_HOSTNAME", "smtp.local");
  let smtp_username = extract_from_env("SMTP_USERNAME", "username");
  let smtp_password = extract_from_env("SMTP_PASSWORD", "password");
  let smtp_sender = extract_from_env("SMTP_SENDER", "sender@whatever.com");

  let oidc_issuer_uri = extract_from_env("OIDC_ISSUER_URI", "https://invalid");

  let redis_url = extract_from_env("REDIS_URL", "redis://valkey:6379/");

  let frontend_dir = Path::new(&frontend_str);

  let postgres_url = format!(
    "postgres://{}:{}@{}/{}",
    postgres_user, postgres_password, postgres_host, postgres_database
  );

  let pool = PgPoolOptions::new()
    .max_connections(5)
    .connect(postgres_url.as_str())
    .await?;

  tracing::info!("Running database migrations...");
  // Run any pending migrations before start
  sqlx::migrate!().run(&pool).await?;
  tracing::info!("Successfully ran all pending migrations");

  let webauthn_origin_url = Url::parse(&webauthn_rp_origin).expect("msg");
  let webauthn_builder =
    WebauthnBuilder::new(&webauthn_rp_id, &webauthn_origin_url).expect("Invalid webauthn config!");
  let webauthn = webauthn_builder.build().expect("Invalid webauthn config!");

  let mailer = if smtp_enabled != "0" {
    let credentials = Credentials::new(smtp_username, smtp_password);
    let transport = Arc::new(
      AsyncSmtpTransport::<lettre::Tokio1Executor>::relay(smtp_hostname.as_str())
        .unwrap()
        .credentials(credentials)
        .build(),
    );
    Some(AppMailer {
      transport,
      sender: smtp_sender,
    })
  } else {
    None
  };

  let redis = redis::Client::open(redis_url).unwrap();
  let redis_connection = redis.get_multiplexed_async_connection().await.unwrap();

  let state = AppState {
    pool,
    private_keys: load_keys(key_dir)?,
    webauthn,
    mailer,
    oidc_issuer_uri,
    redis_connection,
  };

  let cli_args: Vec<String> = env::args().collect();
  if cli_args.len() == 0 {
    panic!("No command line arguments provided! Valid options: serve, setup, send-login-link");
  }

  match cli_args[1].as_str() {
    "serve" => { /* fallthrough */},
    "setup" => {
      handle_setup_cli(&state).await;
      return Ok(());
    },
    "get-login-link" => {
      handle_email_cli(&state).await;
      return Ok(());
    }
    _ => {
      panic!("Invalid command line arguments! Valid options: serve, setup, send-login-link");
    }
  }

  let cors_origin = extract_from_env("CORS_ORIGIN", "");
  let cors = if cors_origin.is_empty() {
    tower_http::cors::CorsLayer::new()
  } else {
    let origin = cors_origin
      .parse::<http::HeaderValue>()
      .expect("CORS_ORIGIN must be a valid header value (e.g. https://localhost:5173)");
    tower_http::cors::CorsLayer::new()
      .allow_origin(origin)
      .allow_methods([
        Method::GET,
        Method::POST,
        Method::PUT,
        Method::DELETE,
        Method::OPTIONS,
      ])
      .allow_headers([
        http::header::CONTENT_TYPE,
        http::header::AUTHORIZATION,
        http::header::ACCEPT,
      ])
      .allow_credentials(true)
  };

  let app = Router::new()
    .merge(auth::router())
    .merge(user::router())
    .merge(client::router())
    .merge(group::router())
    .merge(oauth::router())
    .merge(frontend::router(frontend_dir.to_path_buf()))
    .route_layer(axum::middleware::from_fn_with_state(
      state.clone(),
      middleware::identity_auth,
    ))
    .layer(cors)
    .with_state(state);

  let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
  tracing::info!("Listening on port 3000");
  axum::serve(listener, app)
    .with_graceful_shutdown(shutdown_signal())
    .await
    .unwrap();

  Ok(())
}
