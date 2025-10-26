//! Node registry module
//!
//! Maintains a registry of all discovered nodes with their full configurations

use ndnm_libs::NodeConfig;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// Information about a registered node
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeInfo {
    /// Unique identifier (typically the node_id_hash from config)
    pub node_id: String,

    /// Full node configuration
    pub config: NodeConfig,

    /// Path to the node's directory
    pub path: PathBuf,

    /// Port assigned to this node (managed by Hermes)
    pub port: u16,

    /// Whether the node is currently running
    pub is_running: bool,
}

/// Registry of all discovered nodes
#[derive(Debug, Clone)]
pub struct NodeRegistry {
    /// Map of node_id to NodeInfo
    nodes: HashMap<String, NodeInfo>,
}

impl NodeRegistry {
    /// Create a new empty registry
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
        }
    }

    /// Register a new node
    ///
    /// # Arguments
    ///
    /// * `node_info` - Node information to register
    ///
    /// # Returns
    ///
    /// * `Ok(())` if registered successfully
    /// * `Err(String)` if node_id already exists
    pub fn register(&mut self, node_info: NodeInfo) -> Result<(), String> {
        let node_id = node_info.node_id.clone();

        if self.nodes.contains_key(&node_id) {
            return Err(format!("Node '{}' already registered", node_id));
        }

        self.nodes.insert(node_id, node_info);
        Ok(())
    }

    /// Get node information by ID
    ///
    /// # Arguments
    ///
    /// * `node_id` - The node identifier
    ///
    /// # Returns
    ///
    /// * `Some(NodeInfo)` if found
    /// * `None` if not found
    pub fn get_node(&self, node_id: &str) -> Option<NodeInfo> {
        self.nodes.get(node_id).cloned()
    }

    /// Get all registered nodes
    pub fn get_all_nodes(&self) -> Vec<NodeInfo> {
        self.nodes.values().cloned().collect()
    }

    /// Count of registered nodes
    pub fn count(&self) -> usize {
        self.nodes.len()
    }

    /// Check if a node exists
    pub fn contains(&self, node_id: &str) -> bool {
        self.nodes.contains_key(node_id)
    }

    /// Update node running status
    pub fn set_node_running(&mut self, node_id: &str, is_running: bool) {
        if let Some(node) = self.nodes.get_mut(node_id) {
            node.is_running = is_running;
        }
    }
}

/// Response structure for GET /nodes/registry
#[derive(Debug, Serialize)]
pub struct NodeRegistryResponse {
    pub nodes: Vec<NodeInfo>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use ndnm_libs::NodeConfig;

    fn create_test_node_info(node_id: &str) -> NodeInfo {
        NodeInfo {
            node_id: node_id.to_string(),
            config: NodeConfig {
                node_id_hash: node_id.to_string(),
                label: "Test Node".to_string(),
                node_type: "test".to_string(),
                sections: vec![],
                input_fields: vec![],
            },
            path: PathBuf::from("/test"),
            port: 3001,
            is_running: false,
        }
    }

    #[test]
    fn test_register_node() {
        let mut registry = NodeRegistry::new();
        let node = create_test_node_info("test_node");

        assert!(registry.register(node).is_ok());
        assert_eq!(registry.count(), 1);
    }

    #[test]
    fn test_duplicate_registration() {
        let mut registry = NodeRegistry::new();
        let node = create_test_node_info("test_node");

        registry.register(node.clone()).unwrap();
        let result = registry.register(node);

        assert!(result.is_err());
    }

    #[test]
    fn test_get_node() {
        let mut registry = NodeRegistry::new();
        let node = create_test_node_info("test_node");

        registry.register(node.clone()).unwrap();

        let retrieved = registry.get_node("test_node");
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().node_id, "test_node");
    }

    #[test]
    fn test_set_node_running() {
        let mut registry = NodeRegistry::new();
        let node = create_test_node_info("test_node");

        registry.register(node).unwrap();
        registry.set_node_running("test_node", true);

        let retrieved = registry.get_node("test_node").unwrap();
        assert!(retrieved.is_running);
    }
}
