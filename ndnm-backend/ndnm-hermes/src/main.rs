//! # NDNM Hermes - The Orchestrator
//!
//! Central control plane for the NDNM system. Responsible for:
//! - Discovering and registering nodes
//! - Managing node lifecycle and communication
//! - Orchestrating graph execution
//! - Managing workspace persistence
//!
//! ## Architecture
//!
//! Hermes acts as the "maestro" of the system:
//! 1. Scans `nodes/` directory for available nodes
//! 2. Parses their `config.yaml` configurations
//! 3. Manages node processes and port allocation
//! 4. Executes graphs by coordinating node execution
//! 5. Handles data flow between nodes
//! 6. Provides API for ndnm-brazil (BFF)

mod discovery;
mod orchestrator;
mod registry;
mod workspace;

use anyhow::Result;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use ndnm_libs::AppError;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::net::TcpListener;
use tower_http::trace::TraceLayer;
use tracing::{info, warn};

use discovery::DiscoveryService;
use orchestrator::{GraphExecutionRequest, Orchestrator};
use registry::NodeRegistry;
use workspace::WorkspaceManager;

/// Main application state shared across all handlers
#[derive(Clone)]
struct AppState {
    /// Registry of discovered nodes
    registry: Arc<NodeRegistry>,
    /// Orchestrator for graph execution
    orchestrator: Arc<Orchestrator>,
    /// Workspace manager for persistence
    workspace_manager: Arc<WorkspaceManager>,
}

// === API Handlers ===

/// Response structure for system health check
#[derive(Debug, Serialize)]
struct SystemHealthResponse {
    /// Overall system status
    status: String,
    /// Hermes service status
    hermes: ServiceStatus,
    /// Status of all registered nodes
    nodes: Vec<NodeHealthStatus>,
}

/// Service status information
#[derive(Debug, Serialize)]
struct ServiceStatus {
    /// Is the service healthy?
    healthy: bool,
    /// Additional message
    #[serde(skip_serializing_if = "Option::is_none")]
    message: Option<String>,
}

/// Health status of a single node
#[derive(Debug, Serialize)]
struct NodeHealthStatus {
    /// Node ID
    node_id: String,
    /// Node label
    label: String,
    /// Port the node is running on
    port: u16,
    /// Is the node responding?
    healthy: bool,
    /// Response time in milliseconds
    #[serde(skip_serializing_if = "Option::is_none")]
    response_time_ms: Option<u64>,
    /// Error message if unhealthy
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
}

/// Handler for GET /health - Simple health check endpoint
async fn health_check() -> StatusCode {
    StatusCode::OK
}

/// Handler for GET /health/all - Complete system health check
///
/// Checks Hermes and all registered nodes
async fn health_check_all(State(state): State<AppState>) -> Json<SystemHealthResponse> {
    info!("Performing system-wide health check");

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .build()
        .unwrap();

    let mut node_statuses = Vec::new();
    let nodes = state.registry.get_all_nodes();

    for node_info in nodes {
        let start = std::time::Instant::now();
        let url = format!("http://localhost:{}/health", node_info.port);

        let status = match client.get(&url).send().await {
            Ok(response) => {
                let elapsed = start.elapsed();
                if response.status().is_success() {
                    NodeHealthStatus {
                        node_id: node_info.node_id.clone(),
                        label: node_info.config.label.clone(),
                        port: node_info.port,
                        healthy: true,
                        response_time_ms: Some(elapsed.as_millis() as u64),
                        error: None,
                    }
                } else {
                    NodeHealthStatus {
                        node_id: node_info.node_id.clone(),
                        label: node_info.config.label.clone(),
                        port: node_info.port,
                        healthy: false,
                        response_time_ms: Some(elapsed.as_millis() as u64),
                        error: Some(format!("HTTP {}", response.status())),
                    }
                }
            }
            Err(e) => {
                warn!("Health check failed for node {}: {}", node_info.node_id, e);
                NodeHealthStatus {
                    node_id: node_info.node_id.clone(),
                    label: node_info.config.label.clone(),
                    port: node_info.port,
                    healthy: false,
                    response_time_ms: None,
                    error: Some(e.to_string()),
                }
            }
        };

        node_statuses.push(status);
    }

    // Determine overall status
    let all_healthy = node_statuses.iter().all(|n| n.healthy);
    let overall_status = if all_healthy { "healthy" } else { "degraded" };

    Json(SystemHealthResponse {
        status: overall_status.to_string(),
        hermes: ServiceStatus {
            healthy: true,
            message: Some("Orchestrator running".to_string()),
        },
        nodes: node_statuses,
    })
}

/// Handler for GET /nodes/registry - Get all registered nodes
///
/// Returns the complete structure of all discovered nodes, including
/// their configurations, sections, slots, and behaviors
async fn get_node_registry(
    State(state): State<AppState>,
) -> Result<Json<registry::NodeRegistryResponse>, AppError> {
    let nodes = state.registry.get_all_nodes();
    Ok(Json(registry::NodeRegistryResponse { nodes }))
}

/// Handler for GET /nodes/{node_id} - Get specific node info
async fn get_node_info(
    State(state): State<AppState>,
    Path(node_id): Path<String>,
) -> Result<Json<registry::NodeInfo>, AppError> {
    let node = state
        .registry
        .get_node(&node_id)
        .ok_or_else(|| AppError::BadRequest(format!("Node '{}' not found", node_id)))?;
    Ok(Json(node))
}

/// Handler for POST /graphs/run - Execute a graph
///
/// Receives a graph definition and orchestrates its execution
async fn execute_graph(
    State(state): State<AppState>,
    Json(request): Json<GraphExecutionRequest>,
) -> Result<Json<orchestrator::GraphExecutionResponse>, AppError> {
    info!("Received graph execution request");
    let result = state.orchestrator.execute_graph(request).await?;
    Ok(Json(result))
}

/// Handler for POST /nexus/save - Save workspace
async fn save_workspace(
    State(state): State<AppState>,
    Json(request): Json<workspace::SaveWorkspaceRequest>,
) -> Result<StatusCode, AppError> {
    info!("Saving workspace: {}", request.name);
    state.workspace_manager.save_workspace(request).await?;
    Ok(StatusCode::OK)
}

/// Handler for GET /nexus/load/{name} - Load workspace
async fn load_workspace(
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> Result<Json<workspace::WorkspaceData>, AppError> {
    info!("Loading workspace: {}", name);
    let data = state.workspace_manager.load_workspace(&name).await?;
    Ok(Json(data))
}

/// Handler for GET /nexus/list - List all workspaces
async fn list_workspaces(
    State(state): State<AppState>,
) -> Result<Json<workspace::WorkspaceListResponse>, AppError> {
    let workspaces = state.workspace_manager.list_workspaces().await?;
    Ok(Json(workspace::WorkspaceListResponse { workspaces }))
}

/// Create the main HTTP router with all endpoints
fn create_router(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health_check))
        .route("/health/all", get(health_check_all))
        .route("/nodes/registry", get(get_node_registry))
        .route("/nodes/:node_id", get(get_node_info))
        .route("/graphs/run", post(execute_graph))
        .route("/nexus/save", post(save_workspace))
        .route("/nexus/load/:name", get(load_workspace))
        .route("/nexus/list", get(list_workspaces))
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "ndnm_hermes=info,tower_http=info".into()),
        )
        .init();

    info!("Starting NDNM Hermes - The Orchestrator");

    // Initialize discovery service
    let discovery_service = DiscoveryService::new("./nodes");
    info!("Discovering nodes in ./nodes directory...");

    // Discover and register nodes
    let registry = discovery_service.discover_nodes().await?;
    info!("Discovered {} nodes", registry.count());

    // Print discovered nodes
    for node_info in registry.get_all_nodes() {
        info!(
            "  - {} ({}): {} sections, {} input fields",
            node_info.config.label,
            node_info.node_id,
            node_info.config.sections.len(),
            node_info.config.input_fields.len()
        );
    }

    // Initialize orchestrator
    let orchestrator = Orchestrator::new(Arc::new(registry.clone()));

    // Initialize workspace manager
    let workspace_manager = WorkspaceManager::new("./nexus");

    // Create app state
    let state = AppState {
        registry: Arc::new(registry),
        orchestrator: Arc::new(orchestrator),
        workspace_manager: Arc::new(workspace_manager),
    };

    // Build router
    let app = create_router(state);

    // Get port from environment or use default
    let port = std::env::var("PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(3000);

    let addr = format!("0.0.0.0:{}", port);
    info!("Starting Hermes API server on {}", addr);

    // Start server
    let listener = TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
