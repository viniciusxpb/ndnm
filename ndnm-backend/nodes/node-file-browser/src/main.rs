//! # File Browser Node
//!
//! A filesystem management node that exposes a directory's contents through
//! dynamic I/O slots. Supports copying files into the directory and reading/
//! overwriting existing files.
//!
//! ## Features
//!
//! - Dynamic slot generation based on directory contents
//! - Copy new files into managed directory
//! - Read existing files
//! - Overwrite existing files
//! - Configurable target directory

use anyhow::Result;
use async_trait::async_trait;
use axum::{
    extract::State,
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use ndnm_libs::{load_config, AppError, Node, NodeConfig};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};
use tokio::net::TcpListener;
use tracing::{error, info, warn};
use walkdir::WalkDir;

/// Main node state containing configuration and runtime data
#[derive(Clone)]
struct FileBrowserNode {
    /// Node configuration from config.yaml
    config: NodeConfig,
    /// Current target directory being managed
    target_directory: Arc<RwLock<PathBuf>>,
}

impl FileBrowserNode {
    /// Create a new FileBrowserNode with the given configuration
    ///
    /// # Arguments
    ///
    /// * `config` - Parsed node configuration from config.yaml
    fn new(config: NodeConfig) -> Self {
        Self {
            config,
            target_directory: Arc::new(RwLock::new(PathBuf::from("./managed_files"))),
        }
    }

    /// Set the target directory to manage
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the directory to manage
    ///
    /// # Returns
    ///
    /// * `Ok(())` if directory exists or was created
    /// * `Err(AppError)` if directory cannot be accessed/created
    fn set_target_directory(&self, path: impl AsRef<Path>) -> Result<(), AppError> {
        let path = path.as_ref();

        // Create directory if it doesn't exist
        if !path.exists() {
            fs::create_dir_all(path).map_err(|e| {
                AppError::Internal(format!("Failed to create directory {:?}: {}", path, e))
            })?;
        }

        // Verify it's a directory
        if !path.is_dir() {
            return Err(AppError::BadRequest(format!(
                "{:?} is not a directory",
                path
            )));
        }

        *self.target_directory.write().unwrap() = path.to_path_buf();
        info!("Target directory set to: {:?}", path);
        Ok(())
    }

    /// List all files in the target directory
    ///
    /// # Returns
    ///
    /// * `Ok(Vec<String>)` - List of file names
    /// * `Err(AppError)` - Failed to read directory
    fn list_files(&self) -> Result<Vec<String>, AppError> {
        let dir = self.target_directory.read().unwrap().clone();

        if !dir.exists() {
            return Ok(Vec::new());
        }

        let mut files = Vec::new();

        for entry in WalkDir::new(&dir)
            .max_depth(1)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            if entry.file_type().is_file() {
                if let Some(filename) = entry.file_name().to_str() {
                    files.push(filename.to_string());
                }
            }
        }

        Ok(files)
    }

    /// Copy a file into the target directory
    ///
    /// # Arguments
    ///
    /// * `filename` - Name for the new file
    /// * `content` - File content (as bytes encoded in base64 or direct string)
    ///
    /// # Returns
    ///
    /// * `Ok(PathBuf)` - Path to the created file
    /// * `Err(AppError)` - Failed to write file
    fn copy_file(&self, filename: &str, content: &str) -> Result<PathBuf, AppError> {
        let dir = self.target_directory.read().unwrap().clone();
        let file_path = dir.join(filename);

        fs::write(&file_path, content)
            .map_err(|e| AppError::Internal(format!("Failed to write file: {}", e)))?;

        info!("Copied file to: {:?}", file_path);
        Ok(file_path)
    }

    /// Read a file from the target directory
    ///
    /// # Arguments
    ///
    /// * `filename` - Name of the file to read
    ///
    /// # Returns
    ///
    /// * `Ok(String)` - File contents
    /// * `Err(AppError)` - Failed to read file
    fn read_file(&self, filename: &str) -> Result<String, AppError> {
        let dir = self.target_directory.read().unwrap().clone();
        let file_path = dir.join(filename);

        if !file_path.exists() {
            return Err(AppError::BadRequest(format!(
                "File {:?} does not exist",
                filename
            )));
        }

        fs::read_to_string(&file_path)
            .map_err(|e| AppError::Internal(format!("Failed to read file: {}", e)))
    }

    /// Overwrite an existing file in the target directory
    ///
    /// # Arguments
    ///
    /// * `filename` - Name of the file to overwrite
    /// * `content` - New file content
    ///
    /// # Returns
    ///
    /// * `Ok(())` if successful
    /// * `Err(AppError)` if file doesn't exist or write fails
    fn overwrite_file(&self, filename: &str, content: &str) -> Result<(), AppError> {
        let dir = self.target_directory.read().unwrap().clone();
        let file_path = dir.join(filename);

        if !file_path.exists() {
            return Err(AppError::BadRequest(format!(
                "Cannot overwrite non-existent file: {:?}",
                filename
            )));
        }

        fs::write(&file_path, content)
            .map_err(|e| AppError::Internal(format!("Failed to overwrite file: {}", e)))?;

        info!("Overwrote file: {:?}", file_path);
        Ok(())
    }
}

#[async_trait]
impl Node for FileBrowserNode {
    /// Validate inputs before processing
    ///
    /// Checks that input handles match expected patterns and contain valid data
    fn validate(&self, inputs: &HashMap<String, Value>) -> Result<(), AppError> {
        // Validate that all inputs are either:
        // 1. copy_input_N (for copying new files)
        // 2. internal_input_<filename> (for overwriting existing files)

        for (key, value) in inputs {
            if key.starts_with("copy_input_") {
                // Validate it's a string (file content)
                if !value.is_string() && !value.is_object() {
                    return Err(AppError::BadRequest(format!(
                        "Input '{}' must be a string or object containing file data",
                        key
                    )));
                }
            } else if key.starts_with("internal_input_") {
                // Validate it's a string (file content)
                if !value.is_string() {
                    return Err(AppError::BadRequest(format!(
                        "Input '{}' must be a string (file content)",
                        key
                    )));
                }
            } else {
                warn!("Unknown input handle: {}", key);
            }
        }

        Ok(())
    }

    /// Process node inputs and generate outputs
    ///
    /// Handles both copying new files and reading/overwriting existing files
    async fn process(
        &self,
        inputs: HashMap<String, Value>,
    ) -> Result<HashMap<String, Value>, AppError> {
        let mut outputs = HashMap::new();

        // Process copy_input_N handles (new files being added)
        for (key, value) in inputs.iter() {
            if key.starts_with("copy_input_") {
                // Extract index from key (e.g., "copy_input_0" -> "0")
                let index = key
                    .strip_prefix("copy_input_")
                    .unwrap_or("0");

                // Extract filename and content
                let (filename, content) = if let Some(obj) = value.as_object() {
                    let default_filename = format!("file_{}.txt", index);
                    let filename = obj
                        .get("filename")
                        .and_then(|v| v.as_str())
                        .unwrap_or(&default_filename);
                    let content = obj
                        .get("content")
                        .and_then(|v| v.as_str())
                        .unwrap_or("");
                    (filename.to_string(), content.to_string())
                } else if let Some(content) = value.as_str() {
                    (format!("file_{}.txt", index), content.to_string())
                } else {
                    continue;
                };

                // Copy file
                self.copy_file(&filename, &content)?;

                // Generate corresponding output
                let output_key = format!("copied_output_{}", index);
                outputs.insert(output_key, json!({
                    "filename": filename,
                    "content": content,
                    "status": "copied"
                }));
            }
        }

        // Process internal_input_<filename> handles (overwriting existing files)
        for (key, value) in inputs.iter() {
            if key.starts_with("internal_input_") {
                // Extract filename from key
                let filename = key.strip_prefix("internal_input_").unwrap_or("");

                if let Some(content) = value.as_str() {
                    self.overwrite_file(filename, content)?;

                    // Generate corresponding output (updated file content)
                    let output_key = format!("internal_output_{}", filename);
                    outputs.insert(output_key, Value::String(content.to_string()));
                }
            }
        }

        // Also add outputs for all existing files that weren't modified
        let files = self.list_files()?;
        for filename in files {
            let output_key = format!("internal_output_{}", filename);

            // Only add if not already in outputs (not overwritten)
            if !outputs.contains_key(&output_key) {
                if let Ok(content) = self.read_file(&filename) {
                    outputs.insert(output_key, Value::String(content));
                }
            }
        }

        Ok(outputs)
    }
}

// === HTTP API Handlers ===

/// Request body for the /run endpoint
#[derive(Debug, Deserialize)]
struct RunRequest {
    /// Map of input handle names to their values
    inputs: HashMap<String, Value>,
    /// Optional target directory override
    target_directory: Option<String>,
}

/// Response body for the /run endpoint
#[derive(Debug, Serialize)]
struct RunResponse {
    /// Map of output handle names to their values
    outputs: HashMap<String, Value>,
}

/// Response body for the /list endpoint
#[derive(Debug, Serialize)]
struct ListResponse {
    /// List of files in the target directory
    files: Vec<String>,
}

/// Response body for the /config endpoint
#[derive(Debug, Serialize)]
struct ConfigResponse {
    /// Full node configuration
    config: NodeConfig,
}

/// Handler for GET /health - Health check endpoint
async fn health_check() -> StatusCode {
    StatusCode::OK
}

/// Handler for GET /config - Returns node configuration
async fn get_config(State(node): State<FileBrowserNode>) -> Json<ConfigResponse> {
    Json(ConfigResponse {
        config: node.config.clone(),
    })
}

/// Handler for GET /list - Lists files in target directory
async fn list_files(
    State(node): State<FileBrowserNode>,
) -> Result<Json<ListResponse>, AppError> {
    let files = node.list_files()?;
    Ok(Json(ListResponse { files }))
}

/// Handler for POST /run - Main node execution endpoint
async fn run_node(
    State(node): State<FileBrowserNode>,
    Json(req): Json<RunRequest>,
) -> Result<Json<RunResponse>, AppError> {
    // Update target directory if provided
    if let Some(dir) = req.target_directory {
        node.set_target_directory(&dir)?;
    }

    // Validate inputs
    node.validate(&req.inputs)?;

    // Process
    let outputs = node.process(req.inputs).await?;

    Ok(Json(RunResponse { outputs }))
}

/// Create the Axum router with all endpoints
fn create_router(node: FileBrowserNode) -> Router {
    Router::new()
        .route("/health", get(health_check))
        .route("/config", get(get_config))
        .route("/list", get(list_files))
        .route("/run", post(run_node))
        .with_state(node)
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "node_file_browser=info,tower_http=info".into()),
        )
        .init();

    // Load configuration
    let config = load_config("config.yaml").map_err(|e| {
        error!("Failed to load config.yaml: {}", e);
        e
    })?;

    info!("Loaded node configuration: {}", config.label);

    // Create node instance
    let node = FileBrowserNode::new(config);

    // Initialize default target directory
    node.set_target_directory("./managed_files")?;

    // Build router
    let app = create_router(node);

    // Get port from environment or use default
    let port = std::env::var("PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(3001);

    let addr = format!("0.0.0.0:{}", port);
    info!("Starting node server on {}", addr);

    // Start server
    let listener = TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn create_test_config() -> NodeConfig {
        load_config("config.yaml").unwrap()
    }

    #[test]
    fn test_set_target_directory() {
        let config = create_test_config();
        let node = FileBrowserNode::new(config);

        let temp_dir = TempDir::new().unwrap();
        assert!(node.set_target_directory(temp_dir.path()).is_ok());
    }

    #[test]
    fn test_copy_file() {
        let config = create_test_config();
        let node = FileBrowserNode::new(config);

        let temp_dir = TempDir::new().unwrap();
        node.set_target_directory(temp_dir.path()).unwrap();

        let result = node.copy_file("test.txt", "hello world");
        assert!(result.is_ok());

        let files = node.list_files().unwrap();
        assert!(files.contains(&"test.txt".to_string()));
    }

    #[test]
    fn test_read_file() {
        let config = create_test_config();
        let node = FileBrowserNode::new(config);

        let temp_dir = TempDir::new().unwrap();
        node.set_target_directory(temp_dir.path()).unwrap();

        node.copy_file("test.txt", "hello world").unwrap();

        let content = node.read_file("test.txt").unwrap();
        assert_eq!(content, "hello world");
    }

    #[tokio::test]
    async fn test_process_copy() {
        let config = create_test_config();
        let node = FileBrowserNode::new(config);

        let temp_dir = TempDir::new().unwrap();
        node.set_target_directory(temp_dir.path()).unwrap();

        let mut inputs = HashMap::new();
        inputs.insert(
            "copy_input_0".to_string(),
            json!({"filename": "test.txt", "content": "hello"}),
        );

        let result = node.process(inputs).await;
        assert!(result.is_ok());

        let outputs = result.unwrap();
        assert!(outputs.contains_key("copied_output_0"));
    }
}
