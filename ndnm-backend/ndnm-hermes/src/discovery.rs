//! Node discovery module
//!
//! Scans the filesystem for node directories and loads their configurations

use crate::registry::{NodeInfo, NodeRegistry};
use ndnm_libs::{load_config, AppError};
use std::path::{Path, PathBuf};
use tracing::{error, info, warn};
use walkdir::WalkDir;

/// Service responsible for discovering nodes in the filesystem
pub struct DiscoveryService {
    /// Root directory to scan for nodes
    nodes_dir: PathBuf,
    /// Base port for node allocation
    base_port: u16,
}

impl DiscoveryService {
    /// Create a new discovery service
    ///
    /// # Arguments
    ///
    /// * `nodes_dir` - Path to the directory containing nodes
    pub fn new<P: AsRef<Path>>(nodes_dir: P) -> Self {
        Self {
            nodes_dir: nodes_dir.as_ref().to_path_buf(),
            base_port: 3001, // Start from port 3001 (3000 is for Hermes)
        }
    }

    /// Discover all nodes in the nodes directory
    ///
    /// Scans for directories containing `config.yaml` files and registers them
    ///
    /// # Returns
    ///
    /// * `Ok(NodeRegistry)` - Registry with all discovered nodes
    /// * `Err(AppError)` - Failed to scan directory or read configs
    pub async fn discover_nodes(&self) -> Result<NodeRegistry, AppError> {
        let mut registry = NodeRegistry::new();

        // Check if nodes directory exists
        if !self.nodes_dir.exists() {
            warn!(
                "Nodes directory {:?} does not exist, creating it",
                self.nodes_dir
            );
            std::fs::create_dir_all(&self.nodes_dir).map_err(|e| {
                AppError::Internal(format!("Failed to create nodes directory: {}", e))
            })?;
            return Ok(registry);
        }

        let mut port_counter = 0;

        // Walk through the nodes directory (max depth 1 - only immediate subdirectories)
        for entry in WalkDir::new(&self.nodes_dir)
            .max_depth(1)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            // Skip the root directory itself
            if entry.path() == self.nodes_dir {
                continue;
            }

            // Only process directories
            if !entry.file_type().is_dir() {
                continue;
            }

            // Check for config.yaml in this directory
            let config_path = entry.path().join("config.yaml");
            if !config_path.exists() {
                warn!(
                    "Skipping {:?} - no config.yaml found",
                    entry.file_name()
                );
                continue;
            }

            // Load the configuration
            match load_config(&config_path) {
                Ok(config) => {
                    let node_id = config.node_id_hash.clone();
                    let port = self.base_port + port_counter;
                    port_counter += 1;

                    let node_info = NodeInfo {
                        node_id: node_id.clone(),
                        config,
                        path: entry.path().to_path_buf(),
                        port,
                        is_running: false,
                    };

                    match registry.register(node_info) {
                        Ok(_) => {
                            info!(
                                "Registered node '{}' at {:?} (port {})",
                                node_id,
                                entry.path(),
                                port
                            );
                        }
                        Err(e) => {
                            error!("Failed to register node '{}': {}", node_id, e);
                        }
                    }
                }
                Err(e) => {
                    error!("Failed to load config at {:?}: {}", config_path, e);
                }
            }
        }

        Ok(registry)
    }

    /// Scan for nodes and return their paths
    ///
    /// Utility method to get list of node directories without loading configs
    pub fn scan_node_paths(&self) -> Result<Vec<PathBuf>, AppError> {
        let mut paths = Vec::new();

        if !self.nodes_dir.exists() {
            return Ok(paths);
        }

        for entry in WalkDir::new(&self.nodes_dir)
            .max_depth(1)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            if entry.path() == self.nodes_dir {
                continue;
            }

            if entry.file_type().is_dir() {
                let config_path = entry.path().join("config.yaml");
                if config_path.exists() {
                    paths.push(entry.path().to_path_buf());
                }
            }
        }

        Ok(paths)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_discover_empty_directory() {
        let temp_dir = TempDir::new().unwrap();
        let service = DiscoveryService::new(temp_dir.path());

        let result = service.discover_nodes().await;
        assert!(result.is_ok());

        let registry = result.unwrap();
        assert_eq!(registry.count(), 0);
    }

    #[test]
    fn test_scan_node_paths_empty() {
        let temp_dir = TempDir::new().unwrap();
        let service = DiscoveryService::new(temp_dir.path());

        let result = service.scan_node_paths();
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 0);
    }
}
