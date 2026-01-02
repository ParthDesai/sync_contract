pub mod api;
pub mod config;
pub mod eth;

use axum::{routing::{get, post}, Router};
use tower_http::cors::{AllowOrigin, Any, CorsLayer};

use crate::api::{create_agent_handler, health_handler, rate_handler, AppState};

/// Build the Axum router. Exposed for integration tests.
pub fn build_router(state: AppState) -> Router {
    // Restrict CORS to localhost only (browser Origin header). Requests without an Origin
    // header (typical server-to-server) are unaffected.
    let allow_localhost = AllowOrigin::predicate(|origin, _| {
        let Ok(origin) = origin.to_str() else {
            return false;
        };
        origin == "http://localhost"
            || origin.starts_with("http://localhost:")
            || origin == "http://127.0.0.1"
            || origin.starts_with("http://127.0.0.1:")
    });

    let cors = CorsLayer::new()
        .allow_origin(allow_localhost)
        .allow_methods(Any)
        .allow_headers(Any);

    Router::new()
        .route("/health", get(health_handler))
        .route("/create-agent", post(create_agent_handler))
        .route("/rate", post(rate_handler))
        .layer(cors)
        .with_state(state)
}


