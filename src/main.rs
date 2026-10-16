use axum::{Router, routing::post};
use oracle::pool::Pool;
use tower_http::cors::{Any, CorsLayer};

mod actions;
mod app_config;
mod db;

use app_config::AppConfig;

#[derive(Clone)]
pub struct AppState {
    pub pool: Pool,
}

#[tokio::main]
async fn main() -> Result<(), String> {
    let config = AppConfig::from_env();

    let pool = db::build_pool(&config)?;

    let state = AppState { pool };

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        .route("/query", post(actions::query))
        .route("/execute", post(actions::execute))
        .layer(cors)
        .with_state(state);

    let port = format!("0.0.0.0:{}", config.listen_port);
    println!("Server start at {}", port);
    let listener = tokio::net::TcpListener::bind(port)
        .await
        .map_err(|e| e.to_string())?;

    axum::serve(listener, app)
        .await
        .map_err(|err| err.to_string())?;

    Ok(())
}
