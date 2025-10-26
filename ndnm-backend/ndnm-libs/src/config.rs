//! Configuration structures for NDNM nodes.
//!
//! This module defines all structures needed to parse and work with node
//! configuration files (`config.yaml`). These structures enable dynamic,
//! configuration-driven UI generation in the frontend.

use crate::error::AppError;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

/// Root configuration structure for a node.
///
/// This is the top-level structure parsed from a node's `config.yaml` file.
/// It defines the node's identity, I/O structure, and internal controls.
///
/// # Example YAML
///
/// ```yaml
/// node_id_hash: "hash_sha256_example"
/// label: "Example Node"
/// node_type: "processing"
/// sections:
///   - section_name: "inputs"
///     section_label: "Input Files"
///     behavior: "auto_increment"
///     slot_template:
///       input:
///         name: "file_input"
///         label: "File {index}"
///         type: "FILE_CONTENT"
///         connections: 1
///       output:
///         name: "file_output"
///         label: "Processed File {index}"
///         type: "FILE_CONTENT"
///         connections: "n"
/// input_fields:
///   - name: "setting"
///     label: "Configuration Setting"
///     type: "text"
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeConfig {
    /// Unique deterministic identifier for this node type.
    ///
    /// Should be a hash or unique string (e.g., "hash_sha256_author_nodename")
    pub node_id_hash: String,

    /// Human-readable label for the node (displayed in UI)
    pub label: String,

    /// Functional category of the node (e.g., "filesystem", "processing", "ai")
    pub node_type: String,

    /// List of I/O sections, each with its own behavior and slot templates
    #[serde(default)]
    pub sections: Vec<Section>,

    /// Internal controls/settings for the node
    #[serde(default)]
    pub input_fields: Vec<InputFieldConfig>,
}

/// A section groups related I/O slots with a specific behavior.
///
/// Sections enable dynamic slot generation and organization in the UI.
///
/// # Behaviors
///
/// - `auto_increment`: UI creates new slot pairs as existing ones are connected
/// - `dynamic_per_file`: System generates slot pairs based on files found
/// - `static`: Fixed number of slots defined at config time
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Section {
    /// Internal identifier for this section
    pub section_name: String,

    /// Human-readable label for UI grouping (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub section_label: Option<String>,

    /// Behavior determining how slots are created/managed
    pub behavior: SectionBehavior,

    /// Template defining the paired input/output structure
    pub slot_template: SlotTemplate,
}

/// Behavior type for a section, determining slot creation strategy.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SectionBehavior {
    /// Automatically increment: UI adds more slots when current ones are connected
    AutoIncrement,

    /// Dynamic per file: System creates slot pairs for each file found
    DynamicPerFile,

    /// Static: Fixed slots defined in configuration
    Static,
}

/// Template defining a paired input/output slot structure.
///
/// Each template defines how input and output handles are created and linked.
/// The actual handle names will be generated based on the behavior
/// (e.g., `input_0`, `input_1` for auto_increment).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlotTemplate {
    /// Input slot configuration
    pub input: InputSlotConfig,

    /// Output slot configuration
    pub output: OutputSlotConfig,
}

/// Configuration for an input slot.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InputSlotConfig {
    /// Base name for input handles (e.g., "copy_input" becomes "copy_input_0", "copy_input_1")
    pub name: String,

    /// Display label (can include placeholders like {index}, {filename})
    pub label: String,

    /// Data type expected by this input
    #[serde(rename = "type")]
    pub slot_type: SlotType,

    /// Connection limit: how many wires can connect to each handle
    pub connections: ConnectionCount,
}

/// Configuration for an output slot.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputSlotConfig {
    /// Base name for output handles (e.g., "copied_output" becomes "copied_output_0", "copied_output_1")
    pub name: String,

    /// Display label (can include placeholders like {index}, {filename})
    pub label: String,

    /// Data type provided by this output
    #[serde(rename = "type")]
    pub slot_type: SlotType,

    /// Connection limit: how many wires can connect from each handle
    pub connections: ConnectionCount,
}

/// Data type for slots, defining what kind of data flows through connections.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SlotType {
    /// Binary file content
    FileContent,

    /// Text string
    String,

    /// Numeric value
    Number,

    /// Boolean value
    Boolean,

    /// Arbitrary JSON data
    Json,

    /// Array of values
    Array,

    /// Large binary data (tensors, images)
    Blob,
}

/// Connection count constraint for slots.
///
/// Determines how many connections are allowed to/from a handle.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(untagged)]
pub enum ConnectionCount {
    /// Exact number (typically 1 for single connection)
    Exact(u32),

    /// Unlimited connections (represented as "n" in YAML)
    Unlimited(String),
}

impl ConnectionCount {
    /// Check if this connection count allows unlimited connections
    pub fn is_unlimited(&self) -> bool {
        matches!(self, ConnectionCount::Unlimited(_))
    }

    /// Get the maximum number of connections, or None if unlimited
    pub fn max_connections(&self) -> Option<u32> {
        match self {
            ConnectionCount::Exact(n) => Some(*n),
            ConnectionCount::Unlimited(_) => None,
        }
    }
}

/// Configuration for internal node controls (settings UI).
///
/// These are controls rendered inside the node's UI for configuration,
/// not I/O handles for data flow.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InputFieldConfig {
    /// Internal name/identifier for this field
    pub name: String,

    /// Display label in UI
    pub label: String,

    /// Type of control to render
    #[serde(rename = "type")]
    pub field_type: InputFieldType,

    /// Default value (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default: Option<String>,
}

/// Type of internal control field.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum InputFieldType {
    /// Text input field
    Text,

    /// Numeric input field
    Number,

    /// Checkbox
    Checkbox,

    /// Button/action trigger
    Button,

    /// Dropdown/select
    Select,

    /// File path selector
    FilePath,

    /// Directory path selector
    DirectoryPath,
}

/// Load and parse a node configuration from a YAML file.
///
/// This function reads a `config.yaml` file and parses it into a `NodeConfig` structure,
/// performing validation to ensure all required fields are present.
///
/// # Arguments
///
/// * `config_path` - Path to the config.yaml file
///
/// # Returns
///
/// * `Ok(NodeConfig)` - Successfully parsed configuration
/// * `Err(AppError)` - Failed to read file or parse YAML
///
/// # Example
///
/// ```rust,ignore
/// use ndnm_libs::load_config;
/// use std::path::Path;
///
/// let config = load_config(Path::new("./nodes/my-node/config.yaml"))?;
/// println!("Loaded node: {}", config.label);
/// ```
pub fn load_config<P: AsRef<Path>>(config_path: P) -> Result<NodeConfig, AppError> {
    let path = config_path.as_ref();

    // Read file contents
    let contents = fs::read_to_string(path).map_err(|e| {
        AppError::ConfigError(format!("Failed to read config file at {:?}: {}", path, e))
    })?;

    // Parse YAML
    let config: NodeConfig = serde_yaml::from_str(&contents).map_err(|e| {
        AppError::ConfigError(format!("Failed to parse config YAML at {:?}: {}", path, e))
    })?;

    // Validate required fields
    if config.node_id_hash.is_empty() {
        return Err(AppError::ConfigError(
            "node_id_hash cannot be empty".to_string(),
        ));
    }

    if config.label.is_empty() {
        return Err(AppError::ConfigError("label cannot be empty".to_string()));
    }

    if config.node_type.is_empty() {
        return Err(AppError::ConfigError(
            "node_type cannot be empty".to_string(),
        ));
    }

    Ok(config)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_connection_count_exact() {
        let count = ConnectionCount::Exact(1);
        assert!(!count.is_unlimited());
        assert_eq!(count.max_connections(), Some(1));
    }

    #[test]
    fn test_connection_count_unlimited() {
        let count = ConnectionCount::Unlimited("n".to_string());
        assert!(count.is_unlimited());
        assert_eq!(count.max_connections(), None);
    }

    #[test]
    fn test_section_behavior_serde() {
        let behavior = SectionBehavior::AutoIncrement;
        let yaml = serde_yaml::to_string(&behavior).unwrap();
        assert!(yaml.contains("auto_increment"));

        let parsed: SectionBehavior = serde_yaml::from_str(&yaml).unwrap();
        assert_eq!(parsed, SectionBehavior::AutoIncrement);
    }

    #[test]
    fn test_slot_type_serde() {
        let slot_type = SlotType::FileContent;
        let yaml = serde_yaml::to_string(&slot_type).unwrap();
        assert!(yaml.contains("FILE_CONTENT"));

        let parsed: SlotType = serde_yaml::from_str(&yaml).unwrap();
        assert_eq!(parsed, SlotType::FileContent);
    }
}
