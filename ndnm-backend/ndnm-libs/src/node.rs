//! Node trait definition.
//!
//! This module defines the core `Node` trait that all executable nodes
//! in the NDNM system must implement.

use crate::error::AppError;
use async_trait::async_trait;
use serde_json::Value;
use std::collections::HashMap;

/// Core trait that all NDNM nodes must implement.
///
/// This trait defines the interface for node execution in two phases:
/// 1. **Validation** (`validate`): Quick synchronous validation of inputs
/// 2. **Processing** (`process`): Async execution of the node's main logic
///
/// # Example Implementation
///
/// ```rust,ignore
/// use ndnm_libs::{Node, AppError};
/// use async_trait::async_trait;
/// use serde_json::Value;
/// use std::collections::HashMap;
///
/// struct MyNode;
///
/// #[async_trait]
/// impl Node for MyNode {
///     fn validate(&self, inputs: &HashMap<String, Value>) -> Result<(), AppError> {
///         // Check if required inputs are present
///         if !inputs.contains_key("required_input") {
///             return Err(AppError::BadRequest("Missing required_input".to_string()));
///         }
///         Ok(())
///     }
///
///     async fn process(&self, inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, AppError> {
///         // Perform the actual processing
///         let mut outputs = HashMap::new();
///         outputs.insert("result".to_string(), Value::String("success".to_string()));
///         Ok(outputs)
///     }
/// }
/// ```
#[async_trait]
pub trait Node: Send + Sync {
    /// Validates node inputs before processing.
    ///
    /// This method performs quick, synchronous validation of inputs to catch
    /// errors early without starting expensive async operations.
    ///
    /// # Arguments
    ///
    /// * `inputs` - Map of input handle names to their values (JSON)
    ///
    /// # Returns
    ///
    /// * `Ok(())` if validation passes
    /// * `Err(AppError)` if validation fails
    ///
    /// # Implementation Guidelines
    ///
    /// - Keep this fast and synchronous
    /// - Check for required inputs
    /// - Validate input types and formats
    /// - Don't perform expensive operations here
    /// - Don't access external resources (files, network)
    fn validate(&self, inputs: &HashMap<String, Value>) -> Result<(), AppError>;

    /// Processes the node's main logic asynchronously.
    ///
    /// This method performs the actual work of the node after validation passes.
    /// It can perform expensive operations, access external resources, and take
    /// significant time to complete.
    ///
    /// # Arguments
    ///
    /// * `inputs` - Map of input handle names to their values (JSON)
    ///
    /// # Returns
    ///
    /// * `Ok(HashMap<String, Value>)` - Map of output handle names to their values
    /// * `Err(AppError)` - Processing error
    ///
    /// # Implementation Guidelines
    ///
    /// - This method can be async and take time
    /// - Access files, network, databases as needed
    /// - Return outputs matching the node's config.yaml output handles
    /// - Use descriptive error messages
    /// - Clean up resources on error
    async fn process(
        &self,
        inputs: HashMap<String, Value>,
    ) -> Result<HashMap<String, Value>, AppError>;
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestNode;

    #[async_trait]
    impl Node for TestNode {
        fn validate(&self, inputs: &HashMap<String, Value>) -> Result<(), AppError> {
            if !inputs.contains_key("test_input") {
                return Err(AppError::BadRequest(
                    "Missing test_input".to_string(),
                ));
            }
            Ok(())
        }

        async fn process(
            &self,
            inputs: HashMap<String, Value>,
        ) -> Result<HashMap<String, Value>, AppError> {
            let mut outputs = HashMap::new();
            if let Some(value) = inputs.get("test_input") {
                outputs.insert("test_output".to_string(), value.clone());
            }
            Ok(outputs)
        }
    }

    #[test]
    fn test_validate_success() {
        let node = TestNode;
        let mut inputs = HashMap::new();
        inputs.insert("test_input".to_string(), Value::String("test".to_string()));

        assert!(node.validate(&inputs).is_ok());
    }

    #[test]
    fn test_validate_failure() {
        let node = TestNode;
        let inputs = HashMap::new();

        assert!(node.validate(&inputs).is_err());
    }

    #[tokio::test]
    async fn test_process() {
        let node = TestNode;
        let mut inputs = HashMap::new();
        inputs.insert("test_input".to_string(), Value::String("test".to_string()));

        let result = node.process(inputs).await;
        assert!(result.is_ok());

        let outputs = result.unwrap();
        assert_eq!(
            outputs.get("test_output"),
            Some(&Value::String("test".to_string()))
        );
    }
}
