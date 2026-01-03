use agent_server::api::{AppState};
use agent_server::eth::EthApi;
use agent_server::build_router;
use axum::body::Body;
use axum::http::{Request, StatusCode};
use ethers::types::{Address, TxHash, U256};
use http_body_util::BodyExt;
use serde_json::{json, Value};
use std::sync::{Arc, Mutex};
use tower::ServiceExt;

#[derive(Default)]
struct Calls {
    last_rate: Option<RateCall>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct RateCall {
    data_link: String,
    is_seed_deleted: bool,
    is_valid: bool,
    rating: Option<u8>,
    synthetic_data_link: Option<String>,
    send_tokens_immediately: bool,
}

#[derive(Clone)]
struct MockEth {
    agent: Address,
    calls: Arc<Mutex<Calls>>,
    key: [u8; 32],
    tx: TxHash,
}

#[async_trait::async_trait]
impl EthApi for MockEth {
    fn agent(&self) -> Address {
        self.agent
    }

    async fn submission_key(&self, _data_link: &str) -> anyhow::Result<[u8; 32]> {
        Ok(self.key)
    }

    async fn create_agent(&self) -> anyhow::Result<TxHash> {
        Ok(self.tx)
    }

    async fn user_submission_count(&self, _user: Address) -> anyhow::Result<U256> {
        Ok(U256::from(3u64))
    }

    async fn user_submission_summaries(
        &self,
        _user: Address,
        _start: U256,
        _limit: U256,
    ) -> anyhow::Result<Vec<agent_server::eth::sync_contract::GetUserSubmissionSummariesReturn0>> {
        Ok(vec![agent_server::eth::sync_contract::GetUserSubmissionSummariesReturn0 {
            data_key: [0xAAu8; 32],
            user: Address::repeat_byte(0x11),
            timestamp: U256::from(123u64),
            data_link: "ipfs://bafy".to_string(),
            domain: "ipfs".to_string(),
            data_type: "image".to_string(),
            data_format: "png".to_string(),
            file_size_in_k_b: U256::from(10u64),
            primary_category: [0u8; 32],
            secondary_category: [0u8; 32],
            is_rated: false,
        }])
    }

    async fn rate(
        &self,
        data_link: String,
        is_seed_deleted: bool,
        is_valid: bool,
        rating: Option<u8>,
        synthetic_data_link: Option<String>,
        send_tokens_immediately: bool,
    ) -> anyhow::Result<TxHash> {
        let mut g = self.calls.lock().unwrap();
        g.last_rate = Some(RateCall {
            data_link,
            is_seed_deleted,
            is_valid,
            rating,
            synthetic_data_link,
            send_tokens_immediately,
        });
        Ok(self.tx)
    }
}

async fn json_response(res: axum::response::Response) -> (StatusCode, Value) {
    let status = res.status();
    let body = res.into_body().collect().await.unwrap().to_bytes();
    let v: Value = serde_json::from_slice(&body).unwrap();
    (status, v)
}

#[tokio::test]
async fn rate_rejects_rating_over_100() {
    let calls = Arc::new(Mutex::new(Calls::default()));
    let mock = MockEth {
        agent: Address::repeat_byte(0x11),
        calls,
        key: [0xAA; 32],
        tx: TxHash::from([0xBB; 32]),
    };

    let app = build_router(AppState { eth: Arc::new(mock) });

    let req = Request::builder()
        .method("POST")
        .uri("/rate")
        .header("content-type", "application/json")
        .body(Body::from(
            json!({
              "dataLink": "ipfs://bafy...",
              "isValid": true,
              "rating": 101
            })
            .to_string(),
        ))
        .unwrap();

    let res = app.oneshot(req).await.unwrap();
    let (status, body) = json_response(res).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert!(body["error"].as_str().unwrap().contains("0 and 100"));
}

#[tokio::test]
async fn rate_defaults_and_forwards_params() {
    let calls = Arc::new(Mutex::new(Calls::default()));
    let mock = MockEth {
        agent: Address::repeat_byte(0x22),
        calls: calls.clone(),
        key: [0xCC; 32],
        tx: TxHash::from([0xDD; 32]),
    };

    let app = build_router(AppState { eth: Arc::new(mock) });

    // rating omitted; server should forward rating=None and defaults for seedDeleted/sendTokensImmediately.
    let req = Request::builder()
        .method("POST")
        .uri("/rate")
        .header("content-type", "application/json")
        .body(Body::from(
            json!({
              "dataLink": "ipfs://bafy-agent-test",
              "isValid": true
            })
            .to_string(),
        ))
        .unwrap();

    let res = app.oneshot(req).await.unwrap();
    let (status, body) = json_response(res).await;

    assert_eq!(body["txHash"].as_str().unwrap(), "0xdddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd");
    assert_eq!(body["dataKey"].as_str().unwrap(), "0xcccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc");
    assert_eq!(body["agent"].as_str().unwrap(), "0x2222222222222222222222222222222222222222");

    let last = calls.lock().unwrap().last_rate.clone().unwrap();
    assert_eq!(
        last,
        RateCall {
            data_link: "ipfs://bafy-agent-test".to_string(),
            is_seed_deleted: true,
            is_valid: true,
            rating: None,
            synthetic_data_link: None,
            send_tokens_immediately: false,
        }
    );
}

#[tokio::test]
async fn rate_forwards_synthetic_link_and_flags() {
    let calls = Arc::new(Mutex::new(Calls::default()));
    let mock = MockEth {
        agent: Address::repeat_byte(0x33),
        calls: calls.clone(),
        key: [0xEE; 32],
        tx: TxHash::from([0xFF; 32]),
    };

    let app = build_router(AppState { eth: Arc::new(mock) });

    let req = Request::builder()
        .method("POST")
        .uri("/rate")
        .header("content-type", "application/json")
        .body(Body::from(
            json!({
              "dataLink": "ipfs://bafy-synth",
              "isValid": true,
              "rating": 77,
              "isSeedDeleted": false,
              "syntheticDataLink": "ipfs://bafy-processed",
              "sendTokensImmediately": true
            })
            .to_string(),
        ))
        .unwrap();

    let res = app.oneshot(req).await.unwrap();
    let (status, body) = json_response(res).await;
    assert_eq!(status, StatusCode::OK);

    assert_eq!(
        body["txHash"].as_str().unwrap(),
        "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff"
    );
    assert_eq!(
        body["dataKey"].as_str().unwrap(),
        "0xeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee"
    );
    assert_eq!(
        body["agent"].as_str().unwrap(),
        "0x3333333333333333333333333333333333333333"
    );

    let last = calls.lock().unwrap().last_rate.clone().unwrap();
    assert_eq!(
        last,
        RateCall {
            data_link: "ipfs://bafy-synth".to_string(),
            is_seed_deleted: false,
            is_valid: true,
            rating: Some(77),
            synthetic_data_link: Some("ipfs://bafy-processed".to_string()),
            send_tokens_immediately: true,
        }
    );
}

#[tokio::test]
async fn user_submissions_endpoint_returns_items() {
    let calls = Arc::new(Mutex::new(Calls::default()));
    let mock = MockEth {
        agent: Address::repeat_byte(0x44),
        calls,
        key: [0xAB; 32],
        tx: TxHash::from([0xCD; 32]),
    };

    let app = build_router(AppState { eth: Arc::new(mock) });

    let req = Request::builder()
        .method("GET")
        .uri("/users/0x1111111111111111111111111111111111111111/submissions?start=0&limit=2")
        .body(Body::empty())
        .unwrap();

    let res = app.oneshot(req).await.unwrap();
    let (status, body) = json_response(res).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["total"].as_str().unwrap(), "3");
    assert_eq!(body["items"].as_array().unwrap().len(), 1);
    assert_eq!(body["items"][0]["domain"].as_str().unwrap(), "ipfs");
}


