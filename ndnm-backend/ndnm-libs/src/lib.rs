//! # NDNM Libraries
//!
//! Core library for the NDNM (Node-based Data Network Manipulation) system.
//! This crate provides fundamental types, traits, and utilities used across
//! all NDNM services and nodes.
//!
//! ## Main Components
//!
//! - `Node` trait: Interface that all executable nodes must implement
//! - `AppError`: Standardized error handling
//! - Configuration structures for parsing node `config.yaml` files
//! - Utility functions for config loading and validation

pub mod config;
pub mod error;
pub mod node;

// Re-export main types for convenience
pub use config::{
    load_config, ConnectionCount, InputFieldConfig, InputSlotConfig, NodeConfig, OutputSlotConfig,
    Section, SectionBehavior, SlotTemplate, SlotType,
};
pub use error::AppError;
pub use node::Node;

/// Result type alias using AppError
pub type Result<T> = std::result::Result<T, AppError>;
