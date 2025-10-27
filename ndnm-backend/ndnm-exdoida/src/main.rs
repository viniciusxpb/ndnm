//! # NDNM Exdoida - The Silent Observer
//!
//! Observability service for the NDNM system. Collects logs, metrics, and
//! traces from other services in a highly decoupled manner.
//!
//! ## Design Principles
//!
//! - **Fire-and-forget**: Services send logs via UDP without waiting for response
//! - **Non-blocking**: System continues if Exdoida is down
//! - **Independent**: Does not depend on other services
//! - **Lightweight**: Minimal overhead on the main system

mod storage;
mod udp_server;

use anyhow::Result;
use axum::{
    extract::{Query, State},
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::net::TcpListener;
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use tracing::info;
use chrono::Utc;

use storage::{LogEntry, LogStore};

/// Application state
#[derive(Clone)]
struct AppState {
    /// Log storage
    log_store: Arc<LogStore>,
}

// === API Handlers ===

/// Health check response
#[derive(Debug, Serialize)]
struct HealthResponse {
    status: String,
    service: String,
    logs_count: usize,
}

/// Handler for GET /health
async fn health_check(State(state): State<AppState>) -> Json<HealthResponse> {
    let logs_count = state.log_store.count();

    Json(HealthResponse {
        status: "healthy".to_string(),
        service: "ndnm-exdoida".to_string(),
        logs_count,
    })
}

/// Query parameters for logs endpoint
#[derive(Debug, Deserialize)]
struct LogsQuery {
    /// Limit number of logs returned
    #[serde(default = "default_limit")]
    limit: usize,
    /// Filter by level
    level: Option<String>,
    /// Filter by source service
    source: Option<String>,
}

fn default_limit() -> usize {
    100
}

/// Response for logs endpoint
#[derive(Debug, Serialize)]
struct LogsResponse {
    logs: Vec<LogEntry>,
    total: usize,
}

/// Handler for GET /logs - Get recent logs
async fn get_logs(
    State(state): State<AppState>,
    Query(query): Query<LogsQuery>,
) -> Json<LogsResponse> {
    let mut logs = state.log_store.get_recent(query.limit);

    // Apply filters
    if let Some(level) = &query.level {
        logs.retain(|log| log.level.eq_ignore_ascii_case(level));
    }

    if let Some(source) = &query.source {
        logs.retain(|log| log.source.eq_ignore_ascii_case(source));
    }

    let total = logs.len();

    Json(LogsResponse { logs, total })
}

/// Metrics response
#[derive(Debug, Serialize)]
struct MetricsResponse {
    total_logs: usize,
    logs_by_level: std::collections::HashMap<String, usize>,
    logs_by_source: std::collections::HashMap<String, usize>,
}

/// Handler for GET /metrics - Get metrics
async fn get_metrics(State(state): State<AppState>) -> Json<MetricsResponse> {
    let logs = state.log_store.get_recent(10000); // Get last 10k for metrics

    let mut logs_by_level = std::collections::HashMap::new();
    let mut logs_by_source = std::collections::HashMap::new();

    for log in &logs {
        *logs_by_level.entry(log.level.clone()).or_insert(0) += 1;
        *logs_by_source.entry(log.source.clone()).or_insert(0) += 1;
    }

    Json(MetricsResponse {
        total_logs: state.log_store.count(),
        logs_by_level,
        logs_by_source,
    })
}

/// Handler for DELETE /logs - Clear all logs
async fn clear_logs(State(state): State<AppState>) -> StatusCode {
    state.log_store.clear();
    info!("All logs cleared");
    StatusCode::OK
}

/// Create the HTTP API router
fn create_router(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health_check))
        .route("/logs", get(get_logs))
        .route("/logs", axum::routing::delete(clear_logs))
        .route("/logs", post(create_log))
        .route("/metrics", get(get_metrics))
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "ndnm_exdoida=info,tower_http=info".into()),
        )
        .init();

    info!("Starting NDNM Exdoida - The Silent Observer 👀");

    // Create log store with capacity limit
    let max_logs = std::env::var("MAX_LOGS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(10000);

    let log_store = Arc::new(LogStore::new(max_logs));

    // Create application state
    let state = AppState {
        log_store: log_store.clone(),
    };

    // Start UDP server in background
    let udp_port = std::env::var("UDP_PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(9514); // Default syslog port + 1

    info!("Starting UDP log receiver on port {}", udp_port);
    tokio::spawn(udp_server::start_udp_server(udp_port, log_store));

    // Start HTTP API server
    let http_port = std::env::var("PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(3003);

    let addr = format!("0.0.0.0:{}", http_port);
    info!("Starting Exdoida API server on {}", addr);
    info!("Send logs via UDP to port {}", udp_port);

    let app = create_router(state);
    let listener = TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

/// Handler for POST /logs - Ingest log entry (for frontend/browser)
async fn create_log(State(state): State<AppState>, Json(payload): Json<CreateLogRequest>) -> StatusCode {
    state
        .log_store
        .add(payload.level.clone(), payload.source.clone(), payload.message.clone(), payload.metadata);
    // Also print to console (timestamp + source + colored level)
    print_log(&payload.level, &payload.source, &payload.message);
    StatusCode::OK
}
#[derive(Debug, Deserialize)]
struct CreateLogRequest {
    level: String,
    source: String,
    message: String,
    #[serde(default)]
    metadata: Option<serde_json::Value>,
}

fn color_for_level(level: &str) -> &str {
    match level.to_lowercase().as_str() {
        "info" => "\x1b[32m",   // green
        "warn" => "\x1b[33m",   // yellow
        "error" => "\x1b[31m",  // red
        "debug" => "\x1b[35m",  // magenta
        _ => "\x1b[37m",          // white
    }
}
fn print_log(level: &str, source: &str, message: &str) {
    let ts = Utc::now().to_rfc3339();
    let reset = "\x1b[0m";
    let ts_color = "\x1b[36m"; // cyan
    let src_color = "\x1b[34m"; // blue
    let lvl_color = color_for_level(level);
    println!(
        "{ts_color}[{ts}]{reset} {src_color}[{source}]{reset} {lvl_color}{level}{reset}: {message}",
        ts_color = ts_color,
        ts = ts,
        reset = reset,
        src_color = src_color,
        source = source,
        lvl_color = lvl_color,
        level = level,
        message = message,
    );
}
