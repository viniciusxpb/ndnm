//! # NDNM Brazil - Backend-for-Frontend
//!
//! The BFF layer that bridges the gap between ndnm-argos (frontend) and
//! ndnm-hermes (orchestrator). Provides WebSocket connections for real-time
//! communication and translates frontend commands into Hermes API calls.
//!
//! ## Responsibilities
//!
//! - Maintain persistent WebSocket connections with frontend clients
//! - Relay commands from frontend to Hermes
//! - Broadcast state updates from Hermes to connected clients
//! - Transform data structures between frontend and backend formats
//! - (Future) Authentication and authorization

mod websocket;

use anyhow::Result;
use axum::{
    extract::State,
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use tokio::net::TcpListener;
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use tracing::{info, warn};

/// Main application state shared across handlers
#[derive(Clone)]
struct AppState {
    /// HTTP client for communicating with Hermes
    hermes_client: reqwest::Client,
    /// Base URL for Hermes API
    hermes_url: String,
    /// WebSocket broadcaster for sending updates to clients
    ws_broadcaster: websocket::Broadcaster,
}

impl AppState {
    /// Create a new application state
    fn new(hermes_url: String) -> Self {
        Self {
            hermes_client: reqwest::Client::new(),
            hermes_url,
            ws_broadcaster: websocket::Broadcaster::new(),
        }
    }
}

// === API Handlers ===

/// Health check response
#[derive(Debug, Serialize)]
struct HealthResponse {
    status: String,
    service: String,
    hermes_connected: bool,
}

/// Handler for GET /health
async fn health_check(State(state): State<AppState>) -> Json<HealthResponse> {
    // Try to check if Hermes is accessible
    let hermes_url = format!("{}/health", state.hermes_url);
    let hermes_connected = state
        .hermes_client
        .get(&hermes_url)
        .send()
        .await
        .map(|r| r.status().is_success())
        .unwrap_or(false);

    Json(HealthResponse {
        status: "healthy".to_string(),
        service: "ndnm-brazil".to_string(),
        hermes_connected,
    })
}

/// Request to get node registry from Hermes
async fn get_node_registry(State(state): State<AppState>) -> Result<Json<serde_json::Value>, StatusCode> {
    let url = format!("{}/nodes/registry", state.hermes_url);

    match state.hermes_client.get(&url).send().await {
        Ok(response) => {
            if response.status().is_success() {
                match response.json::<serde_json::Value>().await {
                    Ok(data) => Ok(Json(data)),
                    Err(e) => {
                        warn!("Failed to parse registry response: {}", e);
                        Err(StatusCode::INTERNAL_SERVER_ERROR)
                    }
                }
            } else {
                warn!("Hermes returned status: {}", response.status());
                Err(StatusCode::BAD_GATEWAY)
            }
        }
        Err(e) => {
            warn!("Failed to connect to Hermes: {}", e);
            Err(StatusCode::BAD_GATEWAY)
        }
    }
}

/// Request to execute a graph
#[derive(Debug, Serialize, Deserialize)]
struct ExecuteGraphRequest {
    graph: serde_json::Value,
}

/// Handler for POST /graphs/run
async fn execute_graph(
    State(state): State<AppState>,
    Json(request): Json<ExecuteGraphRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    info!("Received graph execution request via BFF");

    let url = format!("{}/graphs/run", state.hermes_url);

    // Forward the request to Hermes
    match state
        .hermes_client
        .post(&url)
        .json(&request)
        .send()
        .await
    {
        Ok(response) => {
            if response.status().is_success() {
                match response.json::<serde_json::Value>().await {
                    Ok(data) => {
                        // Broadcast execution update to WebSocket clients
                        state
                            .ws_broadcaster
                            .broadcast_json(&serde_json::json!({
                                "type": "graph_execution_complete",
                                "data": data
                            }))
                            .await;

                        Ok(Json(data))
                    }
                    Err(e) => {
                        warn!("Failed to parse execution response: {}", e);
                        Err(StatusCode::INTERNAL_SERVER_ERROR)
                    }
                }
            } else {
                warn!("Hermes returned error status: {}", response.status());
                Err(StatusCode::BAD_GATEWAY)
            }
        }
        Err(e) => {
            warn!("Failed to connect to Hermes: {}", e);
            Err(StatusCode::BAD_GATEWAY)
        }
    }
}

/// Handler for POST /nexus/save
async fn save_workspace(
    State(state): State<AppState>,
    Json(request): Json<serde_json::Value>,
) -> Result<StatusCode, StatusCode> {
    let url = format!("{}/nexus/save", state.hermes_url);

    match state
        .hermes_client
        .post(&url)
        .json(&request)
        .send()
        .await
    {
        Ok(response) => {
            if response.status().is_success() {
                Ok(StatusCode::OK)
            } else {
                Err(StatusCode::BAD_GATEWAY)
            }
        }
        Err(_) => Err(StatusCode::BAD_GATEWAY),
    }
}

/// Handler for GET /nexus/list
async fn list_workspaces(State(state): State<AppState>) -> Result<Json<serde_json::Value>, StatusCode> {
    let url = format!("{}/nexus/list", state.hermes_url);

    match state.hermes_client.get(&url).send().await {
        Ok(response) => {
            if response.status().is_success() {
                match response.json::<serde_json::Value>().await {
                    Ok(data) => Ok(Json(data)),
                    Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
                }
            } else {
                Err(StatusCode::BAD_GATEWAY)
            }
        }
        Err(_) => Err(StatusCode::BAD_GATEWAY),
    }
}

/// Handler for GET /nexus/load/:name
async fn load_workspace(
    State(state): State<AppState>,
    axum::extract::Path(name): axum::extract::Path<String>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let url = format!("{}/nexus/load/{}", state.hermes_url, name);

    match state.hermes_client.get(&url).send().await {
        Ok(response) => {
            if response.status().is_success() {
                match response.json::<serde_json::Value>().await {
                    Ok(data) => Ok(Json(data)),
                    Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
                }
            } else {
                Err(StatusCode::BAD_GATEWAY)
            }
        }
        Err(_) => Err(StatusCode::BAD_GATEWAY),
    }
}

/// Create the main HTTP router
fn create_router(state: AppState) -> Router {
    Router::new()
        // Health check
        .route("/health", get(health_check))
        // WebSocket endpoint
        .route("/ws", get(websocket::websocket_handler))
        // Node registry
        .route("/nodes/registry", get(get_node_registry))
        // Graph execution
        .route("/graphs/run", post(execute_graph))
        // Workspace management
        .route("/nexus/save", post(save_workspace))
        .route("/nexus/list", get(list_workspaces))
        .route("/nexus/load/:name", get(load_workspace))
        // Middleware
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
                .unwrap_or_else(|_| "ndnm_brazil=info,tower_http=info".into()),
        )
        .init();

    info!("Starting NDNM Brazil - Backend-for-Frontend 🇧🇷");

    // Get Hermes URL from environment or use default
    let hermes_url = std::env::var("HERMES_URL")
        .unwrap_or_else(|_| "http://localhost:3000".to_string());

    info!("Configured to connect to Hermes at: {}", hermes_url);

    // Create application state
    let state = AppState::new(hermes_url);

    // Build router
    let app = create_router(state);

    // Get port from environment or use default
    let port = std::env::var("PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(3002);

    let addr = format!("0.0.0.0:{}", port);
    info!("Starting Brazil BFF server on {}", addr);
    info!("WebSocket endpoint: ws://{}/ws", addr);

    // Start server
    let listener = TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
