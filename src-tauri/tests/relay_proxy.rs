use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};

use ai_accounts_hub_lib::relay::credentials::{
    RelayCredentialSource, RelayProvider, RelayProviderCredential,
};
use ai_accounts_hub_lib::relay::proxy::{
    build_relay_router, RelayProxyState, RelayRequestLogEvent, RelayRequestLogger,
};
use axum::body::Body;
use axum::extract::State;
use axum::http::{Method, Request, StatusCode};
use axum::response::IntoResponse;
use axum::routing::any;
use axum::Router;
use serde_json::json;
use tokio::net::TcpListener;

#[derive(Clone)]
struct CapturedRequest {
    method: Method,
    path_and_query: String,
    authorization: Option<String>,
    account_id: Option<String>,
    body: String,
}

#[derive(Clone, Default)]
struct CaptureState {
    captured: Arc<Mutex<Option<CapturedRequest>>>,
}

async fn capture_upstream(
    State(state): State<CaptureState>,
    request: Request<Body>,
) -> impl IntoResponse {
    let method = request.method().clone();
    let path_and_query = request
        .uri()
        .path_and_query()
        .map(|value| value.as_str().to_string())
        .unwrap_or_else(|| request.uri().path().to_string());
    let headers = request.headers().clone();
    let body = axum::body::to_bytes(request.into_body(), 1024 * 1024)
        .await
        .expect("request body");
    *state.captured.lock().expect("capture lock") = Some(CapturedRequest {
        method,
        path_and_query,
        authorization: headers
            .get("authorization")
            .and_then(|value| value.to_str().ok())
            .map(str::to_string),
        account_id: headers
            .get("ChatGPT-Account-Id")
            .and_then(|value| value.to_str().ok())
            .map(str::to_string),
        body: String::from_utf8(body.to_vec()).expect("utf8 body"),
    });

    (StatusCode::CREATED, axum::Json(json!({"ok": true})))
}

async fn spawn_capture_server() -> (String, CaptureState) {
    let state = CaptureState::default();
    let app = Router::new()
        .route("/{*path}", any(capture_upstream))
        .with_state(state.clone());
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind upstream");
    let addr: SocketAddr = listener.local_addr().expect("local addr");
    tokio::spawn(async move {
        axum::serve(listener, app).await.expect("serve upstream");
    });
    (format!("http://{addr}"), state)
}

async fn spawn_sse_server(events: &'static str) -> String {
    async fn sse_response(body: &'static str) -> impl IntoResponse {
        (
            StatusCode::OK,
            [("content-type", "text/event-stream")],
            body,
        )
    }

    let app = Router::new().route(
        "/{*path}",
        any(move || async move { sse_response(events).await }),
    );
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind upstream");
    let addr: SocketAddr = listener.local_addr().expect("local addr");
    tokio::spawn(async move {
        axum::serve(listener, app).await.expect("serve upstream");
    });
    format!("http://{addr}")
}

async fn spawn_status_server(status: StatusCode, body: &'static str) -> String {
    async fn status_response(status: StatusCode, body: &'static str) -> impl IntoResponse {
        (status, [("content-type", "application/json")], body)
    }

    let app = Router::new().route(
        "/{*path}",
        any(move || async move { status_response(status, body).await }),
    );
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind upstream");
    let addr: SocketAddr = listener.local_addr().expect("local addr");
    tokio::spawn(async move {
        axum::serve(listener, app).await.expect("serve upstream");
    });
    format!("http://{addr}")
}

struct StaticCredentialSource {
    credentials: HashMap<RelayProvider, RelayProviderCredential>,
}

impl RelayCredentialSource for StaticCredentialSource {
    fn credential_for(&self, provider: RelayProvider) -> Result<RelayProviderCredential, String> {
        self.credentials
            .get(&provider)
            .cloned()
            .ok_or_else(|| "missing credential".to_string())
    }
}

#[derive(Clone, Default)]
struct CapturingLogger {
    events: Arc<Mutex<Vec<RelayRequestLogEvent>>>,
}

impl RelayRequestLogger for CapturingLogger {
    fn log(&self, event: &RelayRequestLogEvent) {
        self.events.lock().expect("logger lock").push(event.clone());
    }
}

#[tokio::test]
async fn codex_route_forwards_method_path_query_body_and_auth_headers() {
    let (upstream, capture) = spawn_capture_server().await;
    let source = StaticCredentialSource {
        credentials: HashMap::from([(
            RelayProvider::Codex,
            RelayProviderCredential {
                provider: RelayProvider::Codex,
                upstream_base_url: format!("{upstream}/backend-api"),
                bearer_token: "codex-token".to_string(),
                extra_headers: vec![("ChatGPT-Account-Id".to_string(), "acct_123".to_string())],
            },
        )]),
    };
    let app = build_relay_router(RelayProxyState::new(Arc::new(source)));
    let listener = TcpListener::bind("127.0.0.1:0").await.expect("bind relay");
    let relay_addr = listener.local_addr().expect("relay addr");
    tokio::spawn(async move {
        axum::serve(listener, app).await.expect("serve relay");
    });

    let response = reqwest::Client::new()
        .post(format!(
            "http://{relay_addr}/codex/api/codex/responses?stream=true"
        ))
        .header("Authorization", "Bearer caller-token")
        .body(r#"{"model":"gpt-test"}"#)
        .send()
        .await
        .expect("relay response");

    assert_eq!(response.status(), StatusCode::CREATED);
    let captured = capture
        .captured
        .lock()
        .expect("capture lock")
        .clone()
        .expect("captured request");
    assert_eq!(captured.method, Method::POST);
    assert_eq!(
        captured.path_and_query,
        "/backend-api/codex/responses?stream=true"
    );
    assert_eq!(
        captured.authorization.as_deref(),
        Some("Bearer codex-token")
    );
    assert_eq!(captured.account_id.as_deref(), Some("acct_123"));
    assert_eq!(captured.body, r#"{"model":"gpt-test"}"#);
}

#[tokio::test]
async fn codex_usage_route_maps_to_wham_usage() {
    let (upstream, capture) = spawn_capture_server().await;
    let source = StaticCredentialSource {
        credentials: HashMap::from([(
            RelayProvider::Codex,
            RelayProviderCredential {
                provider: RelayProvider::Codex,
                upstream_base_url: format!("{upstream}/backend-api"),
                bearer_token: "codex-token".to_string(),
                extra_headers: Vec::new(),
            },
        )]),
    };
    let app = build_relay_router(RelayProxyState::new(Arc::new(source)));
    let listener = TcpListener::bind("127.0.0.1:0").await.expect("bind relay");
    let relay_addr = listener.local_addr().expect("relay addr");
    tokio::spawn(async move {
        axum::serve(listener, app).await.expect("serve relay");
    });

    let response = reqwest::Client::new()
        .get(format!("http://{relay_addr}/codex/api/codex/usage"))
        .send()
        .await
        .expect("relay response");

    assert_eq!(response.status(), StatusCode::CREATED);
    let captured = capture
        .captured
        .lock()
        .expect("capture lock")
        .clone()
        .expect("captured request");
    assert_eq!(captured.method, Method::GET);
    assert_eq!(captured.path_and_query, "/backend-api/wham/usage");
}

#[tokio::test]
async fn codex_models_route_maps_to_backend_codex_models() {
    let (upstream, capture) = spawn_capture_server().await;
    let source = StaticCredentialSource {
        credentials: HashMap::from([(
            RelayProvider::Codex,
            RelayProviderCredential {
                provider: RelayProvider::Codex,
                upstream_base_url: format!("{upstream}/backend-api"),
                bearer_token: "codex-token".to_string(),
                extra_headers: Vec::new(),
            },
        )]),
    };
    let app = build_relay_router(RelayProxyState::new(Arc::new(source)));
    let listener = TcpListener::bind("127.0.0.1:0").await.expect("bind relay");
    let relay_addr = listener.local_addr().expect("relay addr");
    tokio::spawn(async move {
        axum::serve(listener, app).await.expect("serve relay");
    });

    let response = reqwest::Client::new()
        .get(format!(
            "http://{relay_addr}/codex/api/codex/models?client_version=0.99.0"
        ))
        .send()
        .await
        .expect("relay response");

    assert_eq!(response.status(), StatusCode::CREATED);
    let captured = capture
        .captured
        .lock()
        .expect("capture lock")
        .clone()
        .expect("captured request");
    assert_eq!(captured.method, Method::GET);
    assert_eq!(
        captured.path_and_query,
        "/backend-api/codex/models?client_version=0.99.0"
    );
}

#[tokio::test]
async fn codex_openai_models_route_maps_to_backend_codex_models() {
    let (upstream, capture) = spawn_capture_server().await;
    let source = StaticCredentialSource {
        credentials: HashMap::from([(
            RelayProvider::Codex,
            RelayProviderCredential {
                provider: RelayProvider::Codex,
                upstream_base_url: format!("{upstream}/backend-api"),
                bearer_token: "codex-token".to_string(),
                extra_headers: Vec::new(),
            },
        )]),
    };
    let app = build_relay_router(RelayProxyState::new(Arc::new(source)));
    let listener = TcpListener::bind("127.0.0.1:0").await.expect("bind relay");
    let relay_addr = listener.local_addr().expect("relay addr");
    tokio::spawn(async move {
        axum::serve(listener, app).await.expect("serve relay");
    });

    let response = reqwest::Client::new()
        .get(format!(
            "http://{relay_addr}/codex/v1/models?client_version=0.99.0"
        ))
        .send()
        .await
        .expect("relay response");

    assert_eq!(response.status(), StatusCode::CREATED);
    let captured = capture
        .captured
        .lock()
        .expect("capture lock")
        .clone()
        .expect("captured request");
    assert_eq!(captured.method, Method::GET);
    assert_eq!(
        captured.path_and_query,
        "/backend-api/codex/models?client_version=0.99.0"
    );
}

#[tokio::test]
async fn codex_openai_models_route_adds_default_client_version_when_missing() {
    let (upstream, capture) = spawn_capture_server().await;
    let source = StaticCredentialSource {
        credentials: HashMap::from([(
            RelayProvider::Codex,
            RelayProviderCredential {
                provider: RelayProvider::Codex,
                upstream_base_url: format!("{upstream}/backend-api"),
                bearer_token: "codex-token".to_string(),
                extra_headers: Vec::new(),
            },
        )]),
    };
    let app = build_relay_router(RelayProxyState::new(Arc::new(source)));
    let listener = TcpListener::bind("127.0.0.1:0").await.expect("bind relay");
    let relay_addr = listener.local_addr().expect("relay addr");
    tokio::spawn(async move {
        axum::serve(listener, app).await.expect("serve relay");
    });

    let response = reqwest::Client::new()
        .get(format!("http://{relay_addr}/codex/v1/models"))
        .send()
        .await
        .expect("relay response");

    assert_eq!(response.status(), StatusCode::CREATED);
    let captured = capture
        .captured
        .lock()
        .expect("capture lock")
        .clone()
        .expect("captured request");
    assert_eq!(captured.method, Method::GET);
    assert_eq!(
        captured.path_and_query,
        "/backend-api/codex/models?client_version=0.99.0"
    );
}

#[tokio::test]
async fn codex_openai_models_alias_route_adds_default_client_version_when_missing() {
    let (upstream, capture) = spawn_capture_server().await;
    let source = StaticCredentialSource {
        credentials: HashMap::from([(
            RelayProvider::Codex,
            RelayProviderCredential {
                provider: RelayProvider::Codex,
                upstream_base_url: format!("{upstream}/backend-api"),
                bearer_token: "codex-token".to_string(),
                extra_headers: Vec::new(),
            },
        )]),
    };
    let app = build_relay_router(RelayProxyState::new(Arc::new(source)));
    let listener = TcpListener::bind("127.0.0.1:0").await.expect("bind relay");
    let relay_addr = listener.local_addr().expect("relay addr");
    tokio::spawn(async move {
        axum::serve(listener, app).await.expect("serve relay");
    });

    let response = reqwest::Client::new()
        .get(format!("http://{relay_addr}/codex/models"))
        .send()
        .await
        .expect("relay response");

    assert_eq!(response.status(), StatusCode::CREATED);
    let captured = capture
        .captured
        .lock()
        .expect("capture lock")
        .clone()
        .expect("captured request");
    assert_eq!(captured.method, Method::GET);
    assert_eq!(
        captured.path_and_query,
        "/backend-api/codex/models?client_version=0.99.0"
    );
}

#[tokio::test]
async fn codex_openai_responses_route_maps_to_backend_codex_responses() {
    let (upstream, capture) = spawn_capture_server().await;
    let source = StaticCredentialSource {
        credentials: HashMap::from([(
            RelayProvider::Codex,
            RelayProviderCredential {
                provider: RelayProvider::Codex,
                upstream_base_url: format!("{upstream}/backend-api"),
                bearer_token: "codex-token".to_string(),
                extra_headers: vec![("ChatGPT-Account-Id".to_string(), "acct_123".to_string())],
            },
        )]),
    };
    let app = build_relay_router(RelayProxyState::new(Arc::new(source)));
    let listener = TcpListener::bind("127.0.0.1:0").await.expect("bind relay");
    let relay_addr = listener.local_addr().expect("relay addr");
    tokio::spawn(async move {
        axum::serve(listener, app).await.expect("serve relay");
    });

    let response = reqwest::Client::new()
        .post(format!(
            "http://{relay_addr}/codex/v1/responses?stream=true"
        ))
        .header("content-type", "application/json")
        .body(r#"{"model":"gpt-test","stream":true}"#)
        .send()
        .await
        .expect("relay response");

    assert_eq!(response.status(), StatusCode::CREATED);
    let captured = capture
        .captured
        .lock()
        .expect("capture lock")
        .clone()
        .expect("captured request");
    assert_eq!(captured.method, Method::POST);
    assert_eq!(
        captured.path_and_query,
        "/backend-api/codex/responses?stream=true"
    );
}

#[tokio::test]
async fn codex_openai_responses_alias_route_normalizes_body_for_backend() {
    let (upstream, capture) = spawn_capture_server().await;
    let source = StaticCredentialSource {
        credentials: HashMap::from([(
            RelayProvider::Codex,
            RelayProviderCredential {
                provider: RelayProvider::Codex,
                upstream_base_url: format!("{upstream}/backend-api"),
                bearer_token: "codex-token".to_string(),
                extra_headers: Vec::new(),
            },
        )]),
    };
    let app = build_relay_router(RelayProxyState::new(Arc::new(source)));
    let listener = TcpListener::bind("127.0.0.1:0").await.expect("bind relay");
    let relay_addr = listener.local_addr().expect("relay addr");
    tokio::spawn(async move {
        axum::serve(listener, app).await.expect("serve relay");
    });

    let response = reqwest::Client::new()
        .post(format!("http://{relay_addr}/codex/responses"))
        .header("content-type", "application/json")
        .body(
            r#"{"model":"gpt-test","input":[{"role":"developer","content":"Follow repo rules."},{"role":"user","content":"Say OK"}],"stream":true,"max_output_tokens":16}"#,
        )
        .send()
        .await
        .expect("relay response");

    assert_eq!(response.status(), StatusCode::CREATED);
    let captured = capture
        .captured
        .lock()
        .expect("capture lock")
        .clone()
        .expect("captured request");
    assert_eq!(captured.path_and_query, "/backend-api/codex/responses");
    let body: serde_json::Value = serde_json::from_str(&captured.body).expect("json body");
    assert_eq!(body["instructions"], "You are a helpful coding assistant.");
    assert!(body.get("max_output_tokens").is_none());
    assert_eq!(body["input"][0]["role"], "developer");
    assert_eq!(body["input"][1]["role"], "user");
}

#[tokio::test]
async fn codex_openai_responses_route_normalizes_body_for_backend() {
    let (upstream, capture) = spawn_capture_server().await;
    let source = StaticCredentialSource {
        credentials: HashMap::from([(
            RelayProvider::Codex,
            RelayProviderCredential {
                provider: RelayProvider::Codex,
                upstream_base_url: format!("{upstream}/backend-api"),
                bearer_token: "codex-token".to_string(),
                extra_headers: vec![("ChatGPT-Account-Id".to_string(), "acct_123".to_string())],
            },
        )]),
    };
    let app = build_relay_router(RelayProxyState::new(Arc::new(source)));
    let listener = TcpListener::bind("127.0.0.1:0").await.expect("bind relay");
    let relay_addr = listener.local_addr().expect("relay addr");
    tokio::spawn(async move {
        axum::serve(listener, app).await.expect("serve relay");
    });

    let response = reqwest::Client::new()
        .post(format!("http://{relay_addr}/codex/v1/responses"))
        .header("content-type", "application/json")
        .body(r#"{"model":"gpt-test","input":"Say OK","stream":true}"#)
        .send()
        .await
        .expect("relay response");

    assert_eq!(response.status(), StatusCode::CREATED);
    let captured = capture
        .captured
        .lock()
        .expect("capture lock")
        .clone()
        .expect("captured request");
    let body: serde_json::Value = serde_json::from_str(&captured.body).expect("json body");
    assert_eq!(body["model"], "gpt-test");
    assert_eq!(body["instructions"], "You are a helpful coding assistant.");
    assert_eq!(body["store"], false);
    assert_eq!(body["stream"], true);
    assert_eq!(body["input"][0]["role"], "user");
    assert_eq!(body["input"][0]["content"][0]["type"], "input_text");
    assert_eq!(body["input"][0]["content"][0]["text"], "Say OK");
}

#[tokio::test]
async fn codex_openai_responses_route_extracts_system_messages_to_instructions() {
    let (upstream, capture) = spawn_capture_server().await;
    let source = StaticCredentialSource {
        credentials: HashMap::from([(
            RelayProvider::Codex,
            RelayProviderCredential {
                provider: RelayProvider::Codex,
                upstream_base_url: format!("{upstream}/backend-api"),
                bearer_token: "codex-token".to_string(),
                extra_headers: Vec::new(),
            },
        )]),
    };
    let app = build_relay_router(RelayProxyState::new(Arc::new(source)));
    let listener = TcpListener::bind("127.0.0.1:0").await.expect("bind relay");
    let relay_addr = listener.local_addr().expect("relay addr");
    tokio::spawn(async move {
        axum::serve(listener, app).await.expect("serve relay");
    });

    let response = reqwest::Client::new()
        .post(format!("http://{relay_addr}/codex/v1/responses"))
        .header("content-type", "application/json")
        .body(
            r#"{"model":"gpt-test","stream":true,"input":[{"role":"system","content":[{"type":"text","text":"Follow repo rules."}]},{"role":"user","content":[{"type":"input_text","text":"Say OK"}]}]}"#,
        )
        .send()
        .await
        .expect("relay response");

    assert_eq!(response.status(), StatusCode::CREATED);
    let captured = capture
        .captured
        .lock()
        .expect("capture lock")
        .clone()
        .expect("captured request");
    let body: serde_json::Value = serde_json::from_str(&captured.body).expect("json body");
    assert_eq!(body["instructions"], "Follow repo rules.");
    assert_eq!(body["input"].as_array().map(Vec::len), Some(1));
    assert_eq!(body["input"][0]["role"], "user");
}

#[tokio::test]
async fn codex_openai_responses_route_filters_unsupported_openai_fields() {
    let (upstream, capture) = spawn_capture_server().await;
    let source = StaticCredentialSource {
        credentials: HashMap::from([(
            RelayProvider::Codex,
            RelayProviderCredential {
                provider: RelayProvider::Codex,
                upstream_base_url: format!("{upstream}/backend-api"),
                bearer_token: "codex-token".to_string(),
                extra_headers: Vec::new(),
            },
        )]),
    };
    let app = build_relay_router(RelayProxyState::new(Arc::new(source)));
    let listener = TcpListener::bind("127.0.0.1:0").await.expect("bind relay");
    let relay_addr = listener.local_addr().expect("relay addr");
    tokio::spawn(async move {
        axum::serve(listener, app).await.expect("serve relay");
    });

    let response = reqwest::Client::new()
        .post(format!("http://{relay_addr}/codex/v1/responses"))
        .header("content-type", "application/json")
        .body(
            r#"{"model":"gpt-test","input":"Say OK","stream":true,"max_output_tokens":16,"temperature":0.2}"#,
        )
        .send()
        .await
        .expect("relay response");

    assert_eq!(response.status(), StatusCode::CREATED);
    let captured = capture
        .captured
        .lock()
        .expect("capture lock")
        .clone()
        .expect("captured request");
    let body: serde_json::Value = serde_json::from_str(&captured.body).expect("json body");
    assert!(body.get("max_output_tokens").is_none());
    assert!(body.get("temperature").is_none());
    assert_eq!(body["model"], "gpt-test");
    assert_eq!(body["stream"], true);
}

#[tokio::test]
async fn codex_openai_responses_route_returns_json_when_client_does_not_stream() {
    let upstream = spawn_sse_server(
        "event: response.output_item.done\n\
data: {\"type\":\"response.output_item.done\",\"item\":{\"id\":\"msg_1\",\"type\":\"message\",\"status\":\"completed\",\"content\":[{\"type\":\"output_text\",\"text\":\"OK\",\"annotations\":[],\"logprobs\":[]}],\"role\":\"assistant\"},\"output_index\":0,\"sequence_number\":7}\n\n\
event: response.completed\n\
data: {\"type\":\"response.completed\",\"response\":{\"id\":\"resp_1\",\"object\":\"response\",\"created_at\":1,\"status\":\"completed\",\"completed_at\":1,\"model\":\"gpt-test\",\"output\":[],\"store\":false,\"usage\":{\"input_tokens\":1,\"output_tokens\":1,\"total_tokens\":2}}}\n\n",
    )
    .await;
    let source = StaticCredentialSource {
        credentials: HashMap::from([(
            RelayProvider::Codex,
            RelayProviderCredential {
                provider: RelayProvider::Codex,
                upstream_base_url: format!("{upstream}/backend-api"),
                bearer_token: "codex-token".to_string(),
                extra_headers: Vec::new(),
            },
        )]),
    };
    let app = build_relay_router(RelayProxyState::new(Arc::new(source)));
    let listener = TcpListener::bind("127.0.0.1:0").await.expect("bind relay");
    let relay_addr = listener.local_addr().expect("relay addr");
    tokio::spawn(async move {
        axum::serve(listener, app).await.expect("serve relay");
    });

    let response = reqwest::Client::new()
        .post(format!("http://{relay_addr}/codex/v1/responses"))
        .header("content-type", "application/json")
        .body(r#"{"model":"gpt-test","input":"Say OK","stream":false}"#)
        .send()
        .await
        .expect("relay response");

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        response
            .headers()
            .get("content-type")
            .and_then(|value| value.to_str().ok()),
        Some("application/json")
    );
    let body: serde_json::Value = response.json().await.expect("json response");
    assert_eq!(body["id"], "resp_1");
    assert_eq!(body["output"][0]["id"], "msg_1");
    assert_eq!(body["output"][0]["content"][0]["text"], "OK");
    assert_eq!(body["usage"]["total_tokens"], 2);
}

#[tokio::test]
async fn relay_route_prints_clean_request_log_for_success() {
    let (upstream, _capture) = spawn_capture_server().await;
    let source = StaticCredentialSource {
        credentials: HashMap::from([(
            RelayProvider::Codex,
            RelayProviderCredential {
                provider: RelayProvider::Codex,
                upstream_base_url: format!("{upstream}/backend-api"),
                bearer_token: "secret-codex-token".to_string(),
                extra_headers: vec![("ChatGPT-Account-Id".to_string(), "acct_secret".to_string())],
            },
        )]),
    };
    let logger = CapturingLogger::default();
    let events = logger.events.clone();
    let app = build_relay_router(
        RelayProxyState::new(Arc::new(source)).with_request_logger(Arc::new(logger)),
    );
    let listener = TcpListener::bind("127.0.0.1:0").await.expect("bind relay");
    let relay_addr = listener.local_addr().expect("relay addr");
    tokio::spawn(async move {
        axum::serve(listener, app).await.expect("serve relay");
    });

    let response = reqwest::Client::new()
        .post(format!(
            "http://{relay_addr}/codex/v1/responses?stream=true&key=secret-query-key"
        ))
        .header("Authorization", "Bearer caller-token")
        .header(
            "User-Agent",
            "opencode/1.4.3 ai-sdk/provider-utils/4.0.21 runtime/bun/1.3.11",
        )
        .body(r#"{"model":"gpt-test","input":"do not log me","stream":true}"#)
        .send()
        .await
        .expect("relay response");

    assert_eq!(response.status(), StatusCode::CREATED);
    let events = events.lock().expect("logger lock");
    assert_eq!(events.len(), 1);
    let event = &events[0];
    assert_eq!(event.provider, RelayProvider::Codex);
    assert_eq!(event.method, "POST");
    assert_eq!(
        event.path_and_query,
        "/v1/responses?stream=true&key=[redacted]"
    );
    assert_eq!(event.client.as_deref(), Some("opencode/1.4.3"));
    assert_eq!(event.status, StatusCode::CREATED.as_u16());
    assert_eq!(event.error, None);

    let line = event.to_line();
    assert!(line.contains("provider=codex"));
    assert!(line.contains("method=POST"));
    assert!(line.contains("path=/v1/responses?stream=true&key=[redacted]"));
    assert!(line.contains("status=201"));
    assert!(line.contains("duration_ms="));
    assert!(line.contains("client=opencode/1.4.3"));
    assert!(!line.contains("request="));
    assert!(!line.contains("upstream path="));
    assert!(!line.contains("secret-codex-token"));
    assert!(!line.contains("secret-query-key"));
    assert!(!line.contains("do not log me"));
}

#[tokio::test]
async fn relay_route_keeps_error_log_clean() {
    let upstream = spawn_status_server(
        StatusCode::BAD_REQUEST,
        r#"{"detail":"Instructions are required"}"#,
    )
    .await;
    let source = StaticCredentialSource {
        credentials: HashMap::from([(
            RelayProvider::Codex,
            RelayProviderCredential {
                provider: RelayProvider::Codex,
                upstream_base_url: format!("{upstream}/backend-api"),
                bearer_token: "secret-codex-token".to_string(),
                extra_headers: Vec::new(),
            },
        )]),
    };
    let logger = CapturingLogger::default();
    let events = logger.events.clone();
    let app = build_relay_router(
        RelayProxyState::new(Arc::new(source)).with_request_logger(Arc::new(logger)),
    );
    let listener = TcpListener::bind("127.0.0.1:0").await.expect("bind relay");
    let relay_addr = listener.local_addr().expect("relay addr");
    tokio::spawn(async move {
        axum::serve(listener, app).await.expect("serve relay");
    });

    let response = reqwest::Client::new()
        .post(format!("http://{relay_addr}/codex/responses"))
        .header(
            "User-Agent",
            "opencode/1.4.3 ai-sdk/provider-utils/4.0.21 runtime/bun/1.3.11",
        )
        .header("content-type", "application/json")
        .body(r#"{"model":"gpt-test","input":"do not log me"}"#)
        .send()
        .await
        .expect("relay response");

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let events = events.lock().expect("logger lock");
    assert_eq!(events.len(), 1);
    let event = &events[0];
    let line = event.to_line();
    assert!(line.contains("status=400"));
    assert!(line.contains("client=opencode/1.4.3"));
    assert!(!line.contains("request="));
    assert!(!line.contains("upstream path="));
    assert!(!line.contains("do not log me"));
}

#[tokio::test]
async fn unknown_provider_route_returns_404() {
    let source = StaticCredentialSource {
        credentials: HashMap::new(),
    };
    let app = build_relay_router(RelayProxyState::new(Arc::new(source)));
    let listener = TcpListener::bind("127.0.0.1:0").await.expect("bind relay");
    let relay_addr = listener.local_addr().expect("relay addr");
    tokio::spawn(async move {
        axum::serve(listener, app).await.expect("serve relay");
    });

    let response = reqwest::Client::new()
        .get(format!("http://{relay_addr}/unknown/v1/models"))
        .send()
        .await
        .expect("relay response");

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn claude_route_is_not_registered() {
    let source = StaticCredentialSource {
        credentials: HashMap::new(),
    };
    let app = build_relay_router(RelayProxyState::new(Arc::new(source)));
    let listener = TcpListener::bind("127.0.0.1:0").await.expect("bind relay");
    let relay_addr = listener.local_addr().expect("relay addr");
    tokio::spawn(async move {
        axum::serve(listener, app).await.expect("serve relay");
    });

    let response = reqwest::Client::new()
        .get(format!("http://{relay_addr}/claude/v1/messages"))
        .send()
        .await
        .expect("relay response");

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn gemini_route_is_not_registered() {
    let source = StaticCredentialSource {
        credentials: HashMap::new(),
    };
    let app = build_relay_router(RelayProxyState::new(Arc::new(source)));
    let listener = TcpListener::bind("127.0.0.1:0").await.expect("bind relay");
    let relay_addr = listener.local_addr().expect("relay addr");
    tokio::spawn(async move {
        axum::serve(listener, app).await.expect("serve relay");
    });

    let response = reqwest::Client::new()
        .post(format!(
            "http://{relay_addr}/gemini/v1internal:loadCodeAssist"
        ))
        .send()
        .await
        .expect("relay response");

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}
