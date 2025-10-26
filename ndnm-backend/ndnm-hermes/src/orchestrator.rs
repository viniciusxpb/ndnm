//! Graph orchestration module
//!
//! Coordinates the execution of graphs by managing node execution order,
//! data flow, and error handling

use crate::registry::NodeRegistry;
use ndnm_libs::AppError;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{error, info, warn};
use uuid::Uuid;

/// Request to execute a graph
#[derive(Debug, Clone, Deserialize)]
pub struct GraphExecutionRequest {
    /// Optional execution ID (generated if not provided)
    pub execution_id: Option<String>,

    /// Graph definition containing nodes and connections
    pub graph: GraphDefinition,
}

/// Graph definition structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphDefinition {
    /// List of node instances in the graph
    pub nodes: Vec<GraphNode>,

    /// List of connections between node handles
    pub connections: Vec<Connection>,
}

/// A node instance in a graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphNode {
    /// Unique instance ID for this node in the graph
    pub instance_id: String,

    /// Node type ID (references node_id_hash from registry)
    pub node_type_id: String,

    /// Input field values (internal node settings)
    #[serde(default)]
    pub input_values: HashMap<String, Value>,

    /// Position in UI (optional, for frontend)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub position: Option<Position>,
}

/// Position of a node in the UI
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
    pub x: f64,
    pub y: f64,
}

/// Connection between two node handles
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Connection {
    /// Source node instance ID
    pub from_node: String,

    /// Source handle name (output)
    pub from_handle: String,

    /// Target node instance ID
    pub to_node: String,

    /// Target handle name (input)
    pub to_handle: String,
}

/// Response from graph execution
#[derive(Debug, Serialize)]
pub struct GraphExecutionResponse {
    /// Execution ID
    pub execution_id: String,

    /// Execution status
    pub status: ExecutionStatus,

    /// Results from each node
    pub node_results: HashMap<String, NodeExecutionResult>,

    /// Error message if failed
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// Status of graph execution
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExecutionStatus {
    /// Execution completed successfully
    Success,

    /// Execution failed with error
    Failed,

    /// Execution is still in progress
    InProgress,
}

/// Result from a single node execution
#[derive(Debug, Serialize)]
pub struct NodeExecutionResult {
    /// Node instance ID
    pub instance_id: String,

    /// Execution status
    pub status: String,

    /// Output values from the node
    #[serde(skip_serializing_if = "Option::is_none")]
    pub outputs: Option<HashMap<String, Value>>,

    /// Error message if failed
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// Orchestrator for graph execution
pub struct Orchestrator {
    /// Reference to the node registry
    registry: Arc<NodeRegistry>,

    /// HTTP client for communicating with nodes
    client: reqwest::Client,
}

impl Orchestrator {
    /// Create a new orchestrator
    ///
    /// # Arguments
    ///
    /// * `registry` - Reference to the node registry
    pub fn new(registry: Arc<NodeRegistry>) -> Self {
        Self {
            registry,
            client: reqwest::Client::new(),
        }
    }

    /// Execute a graph
    ///
    /// # Arguments
    ///
    /// * `request` - Graph execution request
    ///
    /// # Returns
    ///
    /// * `Ok(GraphExecutionResponse)` - Execution results
    /// * `Err(AppError)` - Execution failed
    pub async fn execute_graph(
        &self,
        request: GraphExecutionRequest,
    ) -> Result<GraphExecutionResponse, AppError> {
        let execution_id = request
            .execution_id
            .unwrap_or_else(|| Uuid::new_v4().to_string());

        info!("Starting graph execution: {}", execution_id);

        // Validate graph structure
        self.validate_graph(&request.graph)?;

        // Build execution order (topological sort)
        let execution_order = self.build_execution_order(&request.graph)?;

        info!("Execution order: {:?}", execution_order);

        // Execute nodes in order
        let mut node_results = HashMap::new();
        let mut outputs_cache: HashMap<String, HashMap<String, Value>> = HashMap::new();

        for instance_id in execution_order {
            let result = self
                .execute_node(&instance_id, &request.graph, &outputs_cache)
                .await;

            match result {
                Ok((node_result, outputs)) => {
                    outputs_cache.insert(instance_id.clone(), outputs);
                    node_results.insert(instance_id.clone(), node_result);
                }
                Err(e) => {
                    error!("Node {} failed: {}", instance_id, e);

                    let failed_result = NodeExecutionResult {
                        instance_id: instance_id.clone(),
                        status: "failed".to_string(),
                        outputs: None,
                        error: Some(e.to_string()),
                    };

                    node_results.insert(instance_id, failed_result);

                    return Ok(GraphExecutionResponse {
                        execution_id,
                        status: ExecutionStatus::Failed,
                        node_results,
                        error: Some(e.to_string()),
                    });
                }
            }
        }

        info!("Graph execution completed: {}", execution_id);

        Ok(GraphExecutionResponse {
            execution_id,
            status: ExecutionStatus::Success,
            node_results,
            error: None,
        })
    }

    /// Validate graph structure
    ///
    /// Checks that all referenced nodes exist in the registry
    fn validate_graph(&self, graph: &GraphDefinition) -> Result<(), AppError> {
        // Check that all node types exist in registry
        for node in &graph.nodes {
            if !self.registry.contains(&node.node_type_id) {
                return Err(AppError::BadRequest(format!(
                    "Unknown node type: {}",
                    node.node_type_id
                )));
            }
        }

        // Check that all connections reference valid nodes
        let node_ids: Vec<&str> = graph.nodes.iter().map(|n| n.instance_id.as_str()).collect();

        for conn in &graph.connections {
            if !node_ids.contains(&conn.from_node.as_str()) {
                return Err(AppError::BadRequest(format!(
                    "Connection references unknown node: {}",
                    conn.from_node
                )));
            }
            if !node_ids.contains(&conn.to_node.as_str()) {
                return Err(AppError::BadRequest(format!(
                    "Connection references unknown node: {}",
                    conn.to_node
                )));
            }
        }

        Ok(())
    }

    /// Build execution order using topological sort
    ///
    /// Returns a list of instance IDs in execution order
    fn build_execution_order(&self, graph: &GraphDefinition) -> Result<Vec<String>, AppError> {
        // Simple topological sort based on dependencies
        let mut order = Vec::new();
        let mut visited = std::collections::HashSet::new();
        let mut temp_visited = std::collections::HashSet::new();

        // Build dependency map
        let mut dependencies: HashMap<&str, Vec<&str>> = HashMap::new();

        for node in &graph.nodes {
            dependencies.insert(&node.instance_id, Vec::new());
        }

        for conn in &graph.connections {
            dependencies
                .get_mut(conn.to_node.as_str())
                .unwrap()
                .push(&conn.from_node);
        }

        // DFS visit function
        fn visit<'a>(
            node_id: &'a str,
            dependencies: &HashMap<&'a str, Vec<&'a str>>,
            visited: &mut std::collections::HashSet<&'a str>,
            temp_visited: &mut std::collections::HashSet<&'a str>,
            order: &mut Vec<String>,
        ) -> Result<(), AppError> {
            if visited.contains(node_id) {
                return Ok(());
            }

            if temp_visited.contains(node_id) {
                return Err(AppError::BadRequest(
                    "Circular dependency detected in graph".to_string(),
                ));
            }

            temp_visited.insert(node_id);

            if let Some(deps) = dependencies.get(node_id) {
                for &dep in deps {
                    visit(dep, dependencies, visited, temp_visited, order)?;
                }
            }

            temp_visited.remove(node_id);
            visited.insert(node_id);
            order.push(node_id.to_string());

            Ok(())
        }

        // Visit all nodes
        for node in &graph.nodes {
            visit(
                &node.instance_id,
                &dependencies,
                &mut visited,
                &mut temp_visited,
                &mut order,
            )?;
        }

        Ok(order)
    }

    /// Execute a single node
    ///
    /// Gathers inputs from previous node outputs and executes the node
    async fn execute_node(
        &self,
        instance_id: &str,
        graph: &GraphDefinition,
        outputs_cache: &HashMap<String, HashMap<String, Value>>,
    ) -> Result<(NodeExecutionResult, HashMap<String, Value>), AppError> {
        info!("Executing node: {}", instance_id);

        // Find the node in the graph
        let graph_node = graph
            .nodes
            .iter()
            .find(|n| n.instance_id == instance_id)
            .ok_or_else(|| AppError::Internal(format!("Node {} not found in graph", instance_id)))?;

        // Get node info from registry
        let node_info = self
            .registry
            .get_node(&graph_node.node_type_id)
            .ok_or_else(|| {
                AppError::Internal(format!("Node type {} not in registry", graph_node.node_type_id))
            })?;

        // Gather inputs from connections
        let mut inputs = HashMap::new();

        for conn in &graph.connections {
            if conn.to_node == instance_id {
                // This connection provides input to our node
                if let Some(source_outputs) = outputs_cache.get(&conn.from_node) {
                    if let Some(value) = source_outputs.get(&conn.from_handle) {
                        inputs.insert(conn.to_handle.clone(), value.clone());
                    } else {
                        warn!(
                            "Output handle '{}' not found in node '{}'",
                            conn.from_handle, conn.from_node
                        );
                    }
                } else {
                    return Err(AppError::Internal(format!(
                        "Node '{}' has not executed yet, but is a dependency of '{}'",
                        conn.from_node, instance_id
                    )));
                }
            }
        }

        // Call the node's /run endpoint
        let url = format!("http://localhost:{}/run", node_info.port);

        #[derive(Serialize)]
        struct RunRequest {
            inputs: HashMap<String, Value>,
            #[serde(skip_serializing_if = "Option::is_none")]
            target_directory: Option<String>,
        }

        let request_body = RunRequest {
            inputs,
            target_directory: graph_node.input_values.get("target_directory")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
        };

        let response = self
            .client
            .post(&url)
            .json(&request_body)
            .send()
            .await
            .map_err(|e| {
                AppError::Internal(format!("Failed to call node '{}': {}", instance_id, e))
            })?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            return Err(AppError::Internal(format!(
                "Node '{}' returned error: {}",
                instance_id, error_text
            )));
        }

        #[derive(Deserialize)]
        struct RunResponse {
            outputs: HashMap<String, Value>,
        }

        let run_response: RunResponse = response.json().await.map_err(|e| {
            AppError::Internal(format!(
                "Failed to parse response from node '{}': {}",
                instance_id, e
            ))
        })?;

        let node_result = NodeExecutionResult {
            instance_id: instance_id.to_string(),
            status: "success".to_string(),
            outputs: Some(run_response.outputs.clone()),
            error: None,
        };

        Ok((node_result, run_response.outputs))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_graph_empty() {
        let registry = NodeRegistry::new();
        let orchestrator = Orchestrator::new(Arc::new(registry));

        let graph = GraphDefinition {
            nodes: vec![],
            connections: vec![],
        };

        assert!(orchestrator.validate_graph(&graph).is_ok());
    }

    #[test]
    fn test_build_execution_order_simple() {
        let registry = NodeRegistry::new();
        let orchestrator = Orchestrator::new(Arc::new(registry));

        let graph = GraphDefinition {
            nodes: vec![
                GraphNode {
                    instance_id: "node1".to_string(),
                    node_type_id: "type1".to_string(),
                    input_values: HashMap::new(),
                    position: None,
                },
                GraphNode {
                    instance_id: "node2".to_string(),
                    node_type_id: "type2".to_string(),
                    input_values: HashMap::new(),
                    position: None,
                },
            ],
            connections: vec![Connection {
                from_node: "node1".to_string(),
                from_handle: "output".to_string(),
                to_node: "node2".to_string(),
                to_handle: "input".to_string(),
            }],
        };

        let result = orchestrator.build_execution_order(&graph);
        assert!(result.is_ok());

        let order = result.unwrap();
        assert_eq!(order.len(), 2);
        assert_eq!(order[0], "node1");
        assert_eq!(order[1], "node2");
    }
}
