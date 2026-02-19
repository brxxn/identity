// This serves static content so we can deliver the frontend

use std::path::PathBuf;

use axum::{Router, http::StatusCode, response::IntoResponse, routing::get};
use tower_http::services::{ServeDir, ServeFile};

async fn handle_404() -> impl IntoResponse {
  StatusCode::NOT_FOUND
}

async fn handle_robots() -> &'static str {
  "User-Agent: *\nDisallow: /"
}

pub fn router(frontend_dir: PathBuf) -> Router<crate::AppState> {
  Router::new()
    .route("/v1/{*path}", get(handle_404))
    .route("/robots.txt", get(handle_robots))
    .nest_service("/assets", ServeDir::new(frontend_dir.join("assets")))
    .fallback_service(ServeFile::new(frontend_dir.join("index.html")))
}
