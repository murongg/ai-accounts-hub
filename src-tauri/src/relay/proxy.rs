use std::sync::Arc;
use std::time::Instant;

use axum::body::Body;
use axum::extract::{Path, State};
use axum::http::header::{
    AUTHORIZATION, CONNECTION, CONTENT_LENGTH, HOST, TRANSFER_ENCODING, USER_AGENT,
};
use axum::http::{HeaderMap, HeaderName, HeaderValue, Request, StatusCode};
use axum::response::Response;
use axum::routing::any;
use axum::Router;
use futures_util::StreamExt;
use serde_json::{json, Value};

use super::credentials::{RelayCredentialSource, RelayProvider, RelayProviderCredential};

const DEFAULT_CODEX_CLIENT_VERSION: &str = "0.99.0";
const DEFAULT_CODEX_INSTRUCTIONS: &str = "You are a helpful coding assistant.";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum UpstreamResponseMode {
    Passthrough,
    CollectCodexJsonResponse,
}

struct PreparedUpstreamRequest {
    body: Vec<u8>,
    response_mode: UpstreamResponseMode,
}

#[derive(Clone)]
pub struct RelayProxyState {
    credential_source: Arc<dyn RelayCredentialSource>,
    client: reqwest::Client,
    request_logger: Arc<dyn RelayRequestLogger>,
}

impl RelayProxyState {
    pub fn new(credential_source: Arc<dyn RelayCredentialSource>) -> Self {
        Self {
            credential_source,
            client: reqwest::Client::new(),
            request_logger: Arc::new(StderrRelayRequestLogger),
        }
    }

    pub fn with_request_logger(mut self, request_logger: Arc<dyn RelayRequestLogger>) -> Self {
        self.request_logger = request_logger;
        self
    }
}

pub trait RelayRequestLogger: Send + Sync + 'static {
    fn log(&self, event: &RelayRequestLogEvent);
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RelayRequestLogEvent {
    pub provider: RelayProvider,
    pub method: String,
    pub path_and_query: String,
    pub client: Option<String>,
    pub status: u16,
    pub duration_ms: u128,
    pub error: Option<String>,
}

impl RelayRequestLogEvent {
    pub fn to_line(&self) -> String {
        let mut line = format!(
            "[relay] provider={} method={} path={} status={} duration_ms={}",
            provider_log_label(self.provider),
            sanitize_log_value(&self.method),
            sanitize_log_value(&self.path_and_query),
            self.status,
            self.duration_ms
        );
        if let Some(client) = &self.client {
            line.push_str(" client=");
            line.push_str(&sanitize_log_value(client));
        }
        if let Some(error) = &self.error {
            line.push_str(" error=");
            line.push_str(&sanitize_log_value(error));
        }
        line
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct RelayRequestLogContext {
    provider: RelayProvider,
    method: String,
    path_and_query: String,
    client: Option<String>,
}

impl RelayRequestLogContext {
    fn to_event(
        &self,
        status: u16,
        duration_ms: u128,
        error: Option<String>,
    ) -> RelayRequestLogEvent {
        RelayRequestLogEvent {
            provider: self.provider,
            method: self.method.clone(),
            path_and_query: self.path_and_query.clone(),
            client: self.client.clone(),
            status,
            duration_ms,
            error,
        }
    }
}

struct StderrRelayRequestLogger;

impl RelayRequestLogger for StderrRelayRequestLogger {
    fn log(&self, event: &RelayRequestLogEvent) {
        eprintln!("{}", event.to_line());
    }
}

pub fn build_relay_router(state: RelayProxyState) -> Router {
    Router::new()
        .route("/codex/{*path}", any(proxy_codex))
        .with_state(state)
}

async fn proxy_codex(
    State(state): State<RelayProxyState>,
    Path(path): Path<String>,
    request: Request<Body>,
) -> Response {
    proxy_provider(state, RelayProvider::Codex, path, request).await
}

async fn proxy_provider(
    state: RelayProxyState,
    provider: RelayProvider,
    path: String,
    request: Request<Body>,
) -> Response {
    let started_at = Instant::now();
    let method = request.method().as_str().to_string();
    let path_and_query = relay_path_and_query(&path, request.uri().query());
    let request_query = request.uri().query().map(str::to_string);
    let request_method = request.method().clone();
    let inbound_headers = request.headers().clone();
    let body = match axum::body::to_bytes(request.into_body(), usize::MAX).await {
        Ok(body) => body,
        Err(error) => {
            return logged_text_response(
                &state,
                RelayRequestLogContext {
                    provider,
                    method,
                    path_and_query,
                    client: extract_log_client(&inbound_headers),
                },
                started_at,
                StatusCode::BAD_REQUEST,
                error.to_string(),
            );
        }
    };
    let log_context = RelayRequestLogContext {
        provider,
        method: method.clone(),
        path_and_query: path_and_query.clone(),
        client: extract_log_client(&inbound_headers),
    };
    let credential = match state.credential_source.credential_for(provider) {
        Ok(credential) => credential,
        Err(error) => {
            return logged_text_response(
                &state,
                log_context,
                started_at,
                StatusCode::SERVICE_UNAVAILABLE,
                error,
            );
        }
    };
    let upstream_url = build_upstream_url(
        provider,
        &credential.upstream_base_url,
        &path,
        request_query.as_deref(),
    );
    let prepared = match prepare_upstream_request(provider, &path, body.to_vec()) {
        Ok(prepared) => prepared,
        Err(error) => {
            return logged_text_response(
                &state,
                log_context,
                started_at,
                StatusCode::BAD_REQUEST,
                error,
            );
        }
    };

    let mut builder = state.client.request(request_method, upstream_url);
    builder = apply_headers(builder, &inbound_headers, &credential);
    let upstream = match builder.body(prepared.body).send().await {
        Ok(response) => response,
        Err(error) => {
            return logged_text_response(
                &state,
                log_context,
                started_at,
                StatusCode::BAD_GATEWAY,
                error.to_string(),
            );
        }
    };

    let status = upstream.status();
    let headers = upstream.headers().clone();
    log_request(
        &state,
        log_context.to_event(status.as_u16(), started_at.elapsed().as_millis(), None),
    );
    if prepared.response_mode == UpstreamResponseMode::CollectCodexJsonResponse {
        let body = match upstream.bytes().await {
            Ok(body) => body,
            Err(error) => {
                return text_response(StatusCode::BAD_GATEWAY, error.to_string());
            }
        };
        if status.is_success() {
            return match synthesize_codex_json_response(&body) {
                Ok(response_body) => {
                    bytes_response(status, &headers, response_body, Some("application/json"))
                }
                Err(error) => text_response(StatusCode::BAD_GATEWAY, error),
            };
        }
        return bytes_response(status, &headers, body.to_vec(), None);
    }

    let mut response = Response::builder().status(status);
    for (name, value) in &headers {
        if !is_hop_by_hop_header(name) {
            response = response.header(name, value);
        }
    }
    let stream = upstream
        .bytes_stream()
        .map(|chunk| chunk.map_err(std::io::Error::other));
    response
        .body(Body::from_stream(stream))
        .unwrap_or_else(|error| text_response(StatusCode::BAD_GATEWAY, error.to_string()))
}

fn relay_path_and_query(path: &str, query: Option<&str>) -> String {
    let mut path_and_query = format!("/{}", path.trim_start_matches('/'));
    if let Some(query) = query.filter(|value| !value.is_empty()) {
        path_and_query.push('?');
        path_and_query.push_str(&redact_sensitive_query_values(query));
    }
    path_and_query
}

fn build_upstream_url(
    provider: RelayProvider,
    base: &str,
    path: &str,
    query: Option<&str>,
) -> String {
    let upstream_path = rewrite_upstream_path(provider, base, path);
    let upstream_query = rewrite_upstream_query(provider, path, query);
    let mut url = format!(
        "{}/{}",
        base.trim_end_matches('/'),
        upstream_path.trim_start_matches('/')
    );
    if let Some(query) = upstream_query.as_deref().filter(|value| !value.is_empty()) {
        url.push('?');
        url.push_str(query);
    }
    url
}

fn prepare_upstream_request(
    provider: RelayProvider,
    path: &str,
    body: Vec<u8>,
) -> Result<PreparedUpstreamRequest, String> {
    match provider {
        RelayProvider::Codex => prepare_codex_upstream_request(path, body),
        RelayProvider::Claude | RelayProvider::Gemini => Ok(PreparedUpstreamRequest {
            body,
            response_mode: UpstreamResponseMode::Passthrough,
        }),
    }
}

fn prepare_codex_upstream_request(
    path: &str,
    body: Vec<u8>,
) -> Result<PreparedUpstreamRequest, String> {
    if !is_codex_openai_responses_path(path) {
        return Ok(PreparedUpstreamRequest {
            body,
            response_mode: UpstreamResponseMode::Passthrough,
        });
    }

    let mut payload = if body.is_empty() {
        json!({})
    } else {
        serde_json::from_slice::<Value>(&body)
            .map_err(|error| format!("invalid JSON body for Codex /v1/responses: {error}"))?
    };
    let object = payload
        .as_object_mut()
        .ok_or_else(|| "Codex /v1/responses body must be a JSON object".to_string())?;
    let wants_stream = object
        .get("stream")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    object.insert("stream".to_string(), Value::Bool(true));
    object.insert("store".to_string(), Value::Bool(false));
    if !object.contains_key("instructions") || object["instructions"].is_null() {
        object.insert(
            "instructions".to_string(),
            Value::String(DEFAULT_CODEX_INSTRUCTIONS.to_string()),
        );
    } else if object["instructions"]
        .as_str()
        .is_some_and(|value| value.trim().is_empty())
    {
        object.insert(
            "instructions".to_string(),
            Value::String(DEFAULT_CODEX_INSTRUCTIONS.to_string()),
        );
    }
    normalize_codex_openai_input(object);
    extract_system_messages_into_instructions(object);
    retain_supported_codex_response_fields(object);

    let body = serde_json::to_vec(&payload)
        .map_err(|error| format!("failed to encode Codex body: {error}"))?;
    Ok(PreparedUpstreamRequest {
        body,
        response_mode: if wants_stream {
            UpstreamResponseMode::Passthrough
        } else {
            UpstreamResponseMode::CollectCodexJsonResponse
        },
    })
}

fn normalize_codex_openai_input(object: &mut serde_json::Map<String, Value>) {
    let Some(input) = object.get_mut("input") else {
        return;
    };
    if let Some(text) = input.as_str() {
        *input = json!([{
            "role": "user",
            "content": [{
                "type": "input_text",
                "text": text,
            }],
        }]);
    }
}

fn extract_system_messages_into_instructions(object: &mut serde_json::Map<String, Value>) {
    let Some(input) = object.get_mut("input").and_then(Value::as_array_mut) else {
        return;
    };

    let mut system_texts = Vec::new();
    input.retain(|item| {
        let Some(message) = item.as_object() else {
            return true;
        };
        let role = message.get("role").and_then(Value::as_str);
        if role != Some("system") {
            return true;
        }
        if let Some(text) = extract_text_from_content_value(message.get("content")) {
            if !text.trim().is_empty() {
                system_texts.push(text);
            }
        }
        false
    });

    if system_texts.is_empty() {
        return;
    }

    let extracted = system_texts.join("\n\n");
    let current = object
        .get("instructions")
        .and_then(Value::as_str)
        .map(str::trim)
        .unwrap_or("");
    let merged = if current.is_empty() || current == DEFAULT_CODEX_INSTRUCTIONS {
        extracted
    } else {
        format!("{extracted}\n\n{current}")
    };
    object.insert("instructions".to_string(), Value::String(merged));
}

fn extract_text_from_content_value(content: Option<&Value>) -> Option<String> {
    match content {
        Some(Value::String(text)) => Some(text.clone()),
        Some(Value::Array(parts)) => {
            let text = parts
                .iter()
                .filter_map(|part| {
                    let object = part.as_object()?;
                    match object.get("type").and_then(Value::as_str) {
                        Some("text") | Some("input_text") => object
                            .get("text")
                            .and_then(Value::as_str)
                            .map(str::to_string),
                        _ => None,
                    }
                })
                .collect::<Vec<_>>()
                .join("");
            if text.is_empty() {
                None
            } else {
                Some(text)
            }
        }
        _ => None,
    }
}

fn retain_supported_codex_response_fields(object: &mut serde_json::Map<String, Value>) {
    object.retain(|key, _value| {
        matches!(
            key.as_str(),
            "model"
                | "instructions"
                | "input"
                | "tools"
                | "tool_choice"
                | "parallel_tool_calls"
                | "reasoning"
                | "store"
                | "stream"
                | "include"
                | "service_tier"
                | "prompt_cache_key"
                | "text"
        )
    });
}

fn apply_headers(
    mut builder: reqwest::RequestBuilder,
    inbound: &HeaderMap,
    credential: &RelayProviderCredential,
) -> reqwest::RequestBuilder {
    for (name, value) in inbound {
        if should_forward_inbound_header(name) {
            builder = builder.header(name, value);
        }
    }
    builder = builder.header(AUTHORIZATION, format!("Bearer {}", credential.bearer_token));
    for (name, value) in &credential.extra_headers {
        if let (Ok(name), Ok(value)) = (
            HeaderName::from_bytes(name.as_bytes()),
            HeaderValue::from_str(value),
        ) {
            builder = builder.header(name, value);
        }
    }
    if credential.provider == RelayProvider::Claude && !inbound.contains_key("anthropic-version") {
        builder = builder.header("anthropic-version", "2023-06-01");
    }
    builder
}

fn should_forward_inbound_header(name: &HeaderName) -> bool {
    !is_hop_by_hop_header(name)
        && name != HOST
        && name != AUTHORIZATION
        && name.as_str().to_ascii_lowercase() != "x-api-key"
}

fn is_hop_by_hop_header(name: &HeaderName) -> bool {
    name == CONNECTION || name == CONTENT_LENGTH || name == TRANSFER_ENCODING
}

fn text_response(status: StatusCode, text: String) -> Response {
    Response::builder()
        .status(status)
        .header("content-type", "text/plain; charset=utf-8")
        .body(Body::from(text))
        .expect("text response should build")
}

fn bytes_response(
    status: StatusCode,
    headers: &HeaderMap,
    body: Vec<u8>,
    override_content_type: Option<&str>,
) -> Response {
    let mut response = Response::builder().status(status);
    for (name, value) in headers {
        if !is_hop_by_hop_header(name)
            && (override_content_type.is_none()
                || name.as_str().to_ascii_lowercase() != "content-type")
        {
            response = response.header(name, value);
        }
    }
    if let Some(content_type) = override_content_type {
        response = response.header("content-type", content_type);
    }
    response
        .body(Body::from(body))
        .unwrap_or_else(|error| text_response(StatusCode::BAD_GATEWAY, error.to_string()))
}

fn logged_text_response(
    state: &RelayProxyState,
    log_context: RelayRequestLogContext,
    started_at: Instant,
    status: StatusCode,
    text: String,
) -> Response {
    log_request(
        state,
        log_context.to_event(
            status.as_u16(),
            started_at.elapsed().as_millis(),
            Some(text.clone()),
        ),
    );
    text_response(status, text)
}

fn log_request(state: &RelayProxyState, event: RelayRequestLogEvent) {
    state.request_logger.log(&event);
}

fn provider_log_label(provider: RelayProvider) -> &'static str {
    match provider {
        RelayProvider::Codex => "codex",
        RelayProvider::Claude => "claude",
        RelayProvider::Gemini => "gemini",
    }
}

fn sanitize_log_value(value: &str) -> String {
    value
        .chars()
        .map(|value| if value.is_control() { ' ' } else { value })
        .collect()
}

fn extract_log_client(headers: &HeaderMap) -> Option<String> {
    headers
        .get(USER_AGENT)
        .and_then(|value| value.to_str().ok())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(|value| value.split_whitespace().next().unwrap_or(value).to_string())
}

fn rewrite_upstream_path(provider: RelayProvider, base: &str, path: &str) -> String {
    match provider {
        RelayProvider::Codex => rewrite_codex_upstream_path(base, path),
        RelayProvider::Claude | RelayProvider::Gemini => path.trim_start_matches('/').to_string(),
    }
}

fn rewrite_upstream_query(
    provider: RelayProvider,
    path: &str,
    query: Option<&str>,
) -> Option<String> {
    match provider {
        RelayProvider::Codex => rewrite_codex_upstream_query(path, query),
        RelayProvider::Claude | RelayProvider::Gemini => {
            query.filter(|value| !value.is_empty()).map(str::to_string)
        }
    }
}

fn rewrite_codex_upstream_query(path: &str, query: Option<&str>) -> Option<String> {
    let query = query.filter(|value| !value.is_empty());
    if !is_codex_models_request_path(path) {
        return query.map(str::to_string);
    }
    if query_has_key(query, "client_version") {
        return query.map(str::to_string);
    }

    let mut rewritten = String::new();
    if let Some(query) = query {
        rewritten.push_str(query);
        rewritten.push('&');
    }
    rewritten.push_str("client_version=");
    rewritten.push_str(DEFAULT_CODEX_CLIENT_VERSION);
    Some(rewritten)
}

fn is_codex_models_request_path(path: &str) -> bool {
    matches!(
        path.trim_start_matches('/'),
        "v1/models" | "api/codex/models" | "models" | "codex/models"
    )
}

fn is_codex_openai_responses_path(path: &str) -> bool {
    matches!(path.trim_start_matches('/'), "v1/responses" | "responses")
}

fn query_has_key(query: Option<&str>, expected_key: &str) -> bool {
    query
        .into_iter()
        .flat_map(|query| query.split('&'))
        .filter_map(|part| part.split_once('=').map(|(key, _)| key).or(Some(part)))
        .any(|key| key.eq_ignore_ascii_case(expected_key))
}

fn synthesize_codex_json_response(body: &[u8]) -> Result<Vec<u8>, String> {
    let sse =
        std::str::from_utf8(body).map_err(|error| format!("invalid SSE body utf8: {error}"))?;
    let mut completed_response: Option<Value> = None;
    let mut output_items = std::collections::BTreeMap::<usize, Value>::new();

    for chunk in sse.split("\n\n") {
        let data_lines = chunk
            .lines()
            .filter_map(|line| line.strip_prefix("data: "))
            .collect::<Vec<_>>();
        if data_lines.is_empty() {
            continue;
        }
        let payload = data_lines.join("\n");
        let event: Value = serde_json::from_str(&payload)
            .map_err(|error| format!("invalid SSE JSON payload: {error}"))?;
        match event.get("type").and_then(Value::as_str) {
            Some("response.output_item.done") => {
                let Some(item) = event.get("item").cloned() else {
                    continue;
                };
                let index = event
                    .get("output_index")
                    .and_then(Value::as_u64)
                    .unwrap_or(output_items.len() as u64) as usize;
                output_items.insert(index, item);
            }
            Some("response.completed") => {
                completed_response = event.get("response").cloned();
            }
            _ => {}
        }
    }

    let mut response = completed_response
        .ok_or_else(|| "missing response.completed event in Codex SSE stream".to_string())?;
    if let Some(object) = response.as_object_mut() {
        object.insert(
            "output".to_string(),
            Value::Array(output_items.into_values().collect()),
        );
    }
    serde_json::to_vec(&response)
        .map_err(|error| format!("failed to encode Codex JSON response: {error}"))
}

fn rewrite_codex_upstream_path(base: &str, path: &str) -> String {
    let path = path.trim_start_matches('/');
    if base.contains("/backend-api") {
        rewrite_codex_chatgpt_path(path)
    } else {
        rewrite_codex_api_path(path)
    }
}

fn rewrite_codex_chatgpt_path(path: &str) -> String {
    let normalized = path.trim_start_matches('/');
    if let Some(suffix) = normalized.strip_prefix("v1/") {
        return map_codex_openai_suffix_to_chatgpt_path(suffix);
    }
    if let Some(suffix) = normalized.strip_prefix("api/codex/") {
        return map_codex_suffix_to_chatgpt_path(suffix);
    }
    if normalized == "api/codex" {
        return "codex".to_string();
    }
    if normalized.starts_with("codex/") || normalized.starts_with("wham/") {
        return normalized.to_string();
    }
    map_codex_suffix_to_chatgpt_path(normalized)
}

fn map_codex_suffix_to_chatgpt_path(suffix: &str) -> String {
    match suffix {
        "usage" => "wham/usage".to_string(),
        "config/requirements" => "wham/config/requirements".to_string(),
        value if value == "tasks" || value.starts_with("tasks/") => format!("wham/{value}"),
        value
            if value == "models"
                || value.starts_with("models/")
                || value == "responses"
                || value.starts_with("responses/") =>
        {
            format!("codex/{value}")
        }
        value => value.to_string(),
    }
}

fn map_codex_openai_suffix_to_chatgpt_path(suffix: &str) -> String {
    match suffix {
        "models" => "codex/models".to_string(),
        value if value.starts_with("models/") => format!("codex/{value}"),
        "responses" => "codex/responses".to_string(),
        value if value.starts_with("responses/") => format!("codex/{value}"),
        value => map_codex_suffix_to_chatgpt_path(value),
    }
}

fn rewrite_codex_api_path(path: &str) -> String {
    let normalized = path.trim_start_matches('/');
    if let Some(suffix) = normalized.strip_prefix("v1/") {
        return match suffix {
            "models" => "api/codex/models".to_string(),
            value if value.starts_with("models/") => format!("api/codex/{value}"),
            "responses" => "api/codex/responses".to_string(),
            value if value.starts_with("responses/") => format!("api/codex/{value}"),
            value => format!("api/codex/{value}"),
        };
    }
    if normalized == "api/codex" || normalized.starts_with("api/codex/") {
        return normalized.to_string();
    }
    if let Some(suffix) = normalized.strip_prefix("codex/") {
        return format!("api/codex/{suffix}");
    }
    if let Some(suffix) = normalized.strip_prefix("wham/") {
        return format!("api/codex/{suffix}");
    }
    match normalized {
        value
            if value == "usage"
                || value == "config/requirements"
                || value == "tasks"
                || value.starts_with("tasks/")
                || value == "models"
                || value.starts_with("models/")
                || value == "responses"
                || value.starts_with("responses/") =>
        {
            format!("api/codex/{value}")
        }
        value => value.to_string(),
    }
}

fn redact_sensitive_query_values(query: &str) -> String {
    query
        .split('&')
        .map(|part| {
            let Some((key, _value)) = part.split_once('=') else {
                return part.to_string();
            };
            if is_sensitive_query_key(key) {
                format!("{key}=[redacted]")
            } else {
                part.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join("&")
}

fn is_sensitive_query_key(key: &str) -> bool {
    matches!(
        key.to_ascii_lowercase().as_str(),
        "key" | "api_key" | "apikey" | "access_token" | "token" | "auth" | "authorization"
    )
}
