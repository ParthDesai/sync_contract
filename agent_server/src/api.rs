use axum::{extract::State, http::StatusCode, response::{Response, IntoResponse}, Json};
use ethers::types::{Address, TxHash};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use crate::eth::EthApi;
use axum::extract::{Path, Query};
use std::str::FromStr;
use ethers::types::U256;

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

#[derive(Deserialize)]
pub struct PaginationQuery {
    pub start: Option<u64>,
    pub limit: Option<u64>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UserSubmissionsResponse {
    pub user: String,
    pub start: u64,
    pub limit: u64,
    pub total: String,
    pub items: Vec<UserSubmissionItem>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UserSubmissionItem {
    pub data_key: String,
    pub timestamp: String,
    pub data_link: String,
    pub domain: String,
    pub data_type: String,
    pub data_format: String,
    pub file_size_in_kb: String,
    pub primary_category: String,
    pub secondary_category: String,
    pub is_rated: bool,
}

pub async fn user_submissions_handler(
    State(state): State<AppState>,
    Path(user_str): Path<String>,
    Query(q): Query<PaginationQuery>,
) -> Response {
    let user = match Address::from_str(&user_str) {
        Ok(a) => a,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse {
                    error: "invalid user address".to_string(),
                }),
            )
                .into_response();
        }
    };

    let start = q.start.unwrap_or(0);
    let limit = q.limit.unwrap_or(50).min(200);

    let total = match state.eth.user_submission_count(user).await {
        Ok(t) => t,
        Err(e) => {
            return (
                StatusCode::BAD_GATEWAY,
                Json(ErrorResponse {
                    error: format!("failed to query total: {e:#}"),
                }),
            )
                .into_response();
        }
    };

    let rows = match state
        .eth
        .user_submission_summaries(user, U256::from(start), U256::from(limit))
        .await
    {
        Ok(r) => r,
        Err(e) => {
            return (
                StatusCode::BAD_GATEWAY,
                Json(ErrorResponse {
                    error: format!("failed to query submissions: {e:#}"),
                }),
            )
                .into_response();
        }
    };

    let items = rows
        .into_iter()
        .map(|r| {
            let primary_category_string = str::from_utf8(&r.primary_category).unwrap().trim_matches(char::from_u32(0).unwrap()).to_string();
            let secondary_category_string = str::from_utf8(&r.secondary_category).unwrap().trim_matches(char::from_u32(0).unwrap()).to_string();

            UserSubmissionItem {
                data_key: format!("0x{}", hex::encode(r.data_key)),
                timestamp: r.timestamp.to_string(),
                data_link: r.data_link,
                domain: r.domain,
                data_type: r.data_type,
                data_format: r.data_format,
                file_size_in_kb: r.file_size_in_kb.to_string(),
                primary_category: primary_category_string,
                secondary_category: secondary_category_string,
                is_rated: r.is_rated,
            }
        })
        .collect();

    (
        StatusCode::OK,
        Json(UserSubmissionsResponse {
            user: format!("{:#x}", user),
            start,
            limit,
            total: total.to_string(),
            items,
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


