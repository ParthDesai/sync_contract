use axum::{extract::State, http::StatusCode, response::{Response, IntoResponse}, Json};
use ethers::types::{Address, TxHash};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use crate::eth::EthApi;

#[derive(Clone)]
pub struct AppState {
    pub eth: Arc<dyn EthApi>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RateRequest {
    pub data_link: String,
    pub is_valid: bool,
    pub rating: Option<u8>,

    // Optional knobs (default to safe values)
    pub is_seed_deleted: Option<bool>,
    pub synthetic_data_link: Option<String>,
    pub send_tokens_immediately: Option<bool>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RateResponse {
    pub tx_hash: String,
    pub data_key: String,
    pub agent: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ErrorResponse {
    pub error: String,
}

pub async fn health_handler() -> impl IntoResponse {
    (StatusCode::OK, "ok")
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateAgentResponse {
    pub tx_hash: String,
    pub agent: String,
}

pub async fn create_agent_handler(State(state): State<AppState>) -> Response {
    let tx_hash = match state.eth.create_agent().await {
        Ok(h) => format!("{h:#x}"),
        Err(e) => {
            return (
                StatusCode::BAD_GATEWAY,
                Json(ErrorResponse {
                    error: format!("failed to send tx: {e:#}"),
                }),
            )
                .into_response();
        }
    };

    (
        StatusCode::OK,
        Json(CreateAgentResponse {
            tx_hash,
            agent: format!("{:#x}", state.eth.agent()),
        }),
    )
        .into_response()
}

pub async fn rate_handler(
    State(state): State<AppState>,
    Json(req): Json<RateRequest>,
) -> Response {
    // Basic validation at API layer (contract will also enforce most invariants)
    if req.data_link.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: "dataLink must not be empty".to_string(),
            }),
        ).into_response();
    }

    if let Some(r) = req.rating {
        if r > 100 {
            return (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse {
                    error: "rating must be between 0 and 100".to_string(),
                }),
            ).into_response();
        }
    }

    let is_seed_deleted = req.is_seed_deleted.unwrap_or(true);
    let send_tokens_immediately = req.send_tokens_immediately.unwrap_or(false);

    // Compute key for response (view call)
    let data_key = match state.eth.submission_key(&req.data_link).await {
        Ok(k) => format!("0x{}", hex::encode(k)),
        Err(e) => {
            return (
                StatusCode::BAD_GATEWAY,
                Json(ErrorResponse {
                    error: format!("failed to compute submission key: {e:#}"),
                }),
            ).into_response();
        }
    };

    let tx_hash = match state
        .eth
        .rate(
            req.data_link,
            is_seed_deleted,
            req.is_valid,
            req.rating,
            req.synthetic_data_link,
            send_tokens_immediately,
        )
        .await
    {
        Ok(h) => format!("{h:#x}"),
        Err(e) => {
            return (
                StatusCode::BAD_GATEWAY,
                Json(ErrorResponse {
                    error: format!("failed to send tx: {e:#}"),
                }),
            ).into_response();
        }
    };

    (
        StatusCode::OK,
        Json(RateResponse {
            tx_hash,
            data_key,
            agent: format!("{:#x}", state.eth.agent()),
        }),
    ).into_response()
}


