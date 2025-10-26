//! Workspace persistence module
//!
//! Manages saving and loading of workspace data to/from the nexus directory

use ndnm_libs::AppError;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};
use tracing::{info, warn};

/// Workspace manager for persistence
pub struct WorkspaceManager {
    /// Directory where workspaces are stored
    nexus_dir: PathBuf,
}

impl WorkspaceManager {
    /// Create a new workspace manager
    ///
    /// # Arguments
    ///
    /// * `nexus_dir` - Path to the nexus directory
    pub fn new<P: AsRef<Path>>(nexus_dir: P) -> Self {
        let nexus_dir = nexus_dir.as_ref().to_path_buf();

        // Create nexus directory if it doesn't exist
        if !nexus_dir.exists() {
            if let Err(e) = fs::create_dir_all(&nexus_dir) {
                warn!("Failed to create nexus directory: {}", e);
            }
        }

        Self { nexus_dir }
    }

    /// Save a workspace
    ///
    /// # Arguments
    ///
    /// * `request` - Save workspace request
    ///
    /// # Returns
    ///
    /// * `Ok(())` if saved successfully
    /// * `Err(AppError)` if save failed
    pub async fn save_workspace(&self, request: SaveWorkspaceRequest) -> Result<(), AppError> {
        let filename = format!("{}.json", sanitize_filename(&request.name));
        let file_path = self.nexus_dir.join(&filename);

        info!("Saving workspace to: {:?}", file_path);

        // Serialize to JSON
        let json = serde_json::to_string_pretty(&request.data).map_err(|e| {
            AppError::Internal(format!("Failed to serialize workspace data: {}", e))
        })?;

        // Write to file
        fs::write(&file_path, json)
            .map_err(|e| AppError::Internal(format!("Failed to write workspace file: {}", e)))?;

        info!("Workspace '{}' saved successfully", request.name);
        Ok(())
    }

    /// Load a workspace
    ///
    /// # Arguments
    ///
    /// * `name` - Workspace name
    ///
    /// # Returns
    ///
    /// * `Ok(WorkspaceData)` - Loaded workspace data
    /// * `Err(AppError)` - Failed to load
    pub async fn load_workspace(&self, name: &str) -> Result<WorkspaceData, AppError> {
        let filename = format!("{}.json", sanitize_filename(name));
        let file_path = self.nexus_dir.join(&filename);

        if !file_path.exists() {
            return Err(AppError::BadRequest(format!(
                "Workspace '{}' not found",
                name
            )));
        }

        info!("Loading workspace from: {:?}", file_path);

        // Read file
        let contents = fs::read_to_string(&file_path)
            .map_err(|e| AppError::Internal(format!("Failed to read workspace file: {}", e)))?;

        // Parse JSON
        let data: WorkspaceData = serde_json::from_str(&contents)
            .map_err(|e| AppError::Internal(format!("Failed to parse workspace data: {}", e)))?;

        info!("Workspace '{}' loaded successfully", name);
        Ok(data)
    }

    /// List all available workspaces
    ///
    /// # Returns
    ///
    /// * `Ok(Vec<String>)` - List of workspace names
    /// * `Err(AppError)` - Failed to list
    pub async fn list_workspaces(&self) -> Result<Vec<String>, AppError> {
        if !self.nexus_dir.exists() {
            return Ok(Vec::new());
        }

        let mut workspaces = Vec::new();

        let entries = fs::read_dir(&self.nexus_dir).map_err(|e| {
            AppError::Internal(format!("Failed to read nexus directory: {}", e))
        })?;

        for entry in entries {
            if let Ok(entry) = entry {
                let path = entry.path();
                if path.is_file() {
                    if let Some(ext) = path.extension() {
                        if ext == "json" {
                            if let Some(stem) = path.file_stem() {
                                if let Some(name) = stem.to_str() {
                                    workspaces.push(name.to_string());
                                }
                            }
                        }
                    }
                }
            }
        }

        workspaces.sort();
        Ok(workspaces)
    }

    /// Delete a workspace
    ///
    /// # Arguments
    ///
    /// * `name` - Workspace name to delete
    ///
    /// # Returns
    ///
    /// * `Ok(())` if deleted successfully
    /// * `Err(AppError)` if deletion failed
    pub async fn delete_workspace(&self, name: &str) -> Result<(), AppError> {
        let filename = format!("{}.json", sanitize_filename(name));
        let file_path = self.nexus_dir.join(&filename);

        if !file_path.exists() {
            return Err(AppError::BadRequest(format!(
                "Workspace '{}' not found",
                name
            )));
        }

        fs::remove_file(&file_path)
            .map_err(|e| AppError::Internal(format!("Failed to delete workspace file: {}", e)))?;

        info!("Workspace '{}' deleted successfully", name);
        Ok(())
    }
}

/// Request to save a workspace
#[derive(Debug, Deserialize)]
pub struct SaveWorkspaceRequest {
    /// Workspace name
    pub name: String,

    /// Workspace data to save
    pub data: WorkspaceData,
}

/// Workspace data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceData {
    /// Graph definition (nodes and connections)
    pub graph: Value,

    /// Metadata about the workspace
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<WorkspaceMetadata>,
}

/// Metadata about a workspace
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceMetadata {
    /// When the workspace was created
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<String>,

    /// When the workspace was last modified
    #[serde(skip_serializing_if = "Option::is_none")]
    pub modified_at: Option<String>,

    /// User who created the workspace
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_by: Option<String>,

    /// Description of the workspace
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

/// Response for workspace list endpoint
#[derive(Debug, Serialize)]
pub struct WorkspaceListResponse {
    /// List of workspace names
    pub workspaces: Vec<String>,
}

/// Sanitize a filename by removing/replacing invalid characters
fn sanitize_filename(name: &str) -> String {
    name.chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_save_and_load_workspace() {
        let temp_dir = TempDir::new().unwrap();
        let manager = WorkspaceManager::new(temp_dir.path());

        let data = WorkspaceData {
            graph: serde_json::json!({"nodes": [], "connections": []}),
            metadata: Some(WorkspaceMetadata {
                created_at: Some("2024-01-01".to_string()),
                modified_at: None,
                created_by: Some("test_user".to_string()),
                description: Some("Test workspace".to_string()),
            }),
        };

        let request = SaveWorkspaceRequest {
            name: "test_workspace".to_string(),
            data: data.clone(),
        };

        // Save
        let save_result = manager.save_workspace(request).await;
        assert!(save_result.is_ok());

        // Load
        let load_result = manager.load_workspace("test_workspace").await;
        assert!(load_result.is_ok());

        let loaded = load_result.unwrap();
        assert!(loaded.metadata.is_some());
    }

    #[tokio::test]
    async fn test_list_workspaces() {
        let temp_dir = TempDir::new().unwrap();
        let manager = WorkspaceManager::new(temp_dir.path());

        // Initially empty
        let list = manager.list_workspaces().await.unwrap();
        assert_eq!(list.len(), 0);

        // Save a workspace
        let data = WorkspaceData {
            graph: serde_json::json!({"nodes": []}),
            metadata: None,
        };

        let request = SaveWorkspaceRequest {
            name: "workspace1".to_string(),
            data,
        };

        manager.save_workspace(request).await.unwrap();

        // Should now have one workspace
        let list = manager.list_workspaces().await.unwrap();
        assert_eq!(list.len(), 1);
        assert!(list.contains(&"workspace1".to_string()));
    }

    #[test]
    fn test_sanitize_filename() {
        assert_eq!(sanitize_filename("hello world"), "hello_world");
        assert_eq!(sanitize_filename("test/file.txt"), "test_file_txt");
        assert_eq!(sanitize_filename("valid-name_123"), "valid-name_123");
    }
}
