//! # Log Storage
//!
//! Thread-safe in-memory storage for log entries with circular buffer behavior.
//! When the storage reaches max capacity, oldest entries are automatically removed.

use chrono::{DateTime, Utc};
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

/// A single log entry received from a service
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    /// Unique identifier for this log entry
    pub id: usize,
    /// Timestamp when the log was received
    pub timestamp: DateTime<Utc>,
    /// Log level (info, warn, error, debug, trace)
    pub level: String,
    /// Source service that sent the log
    pub source: String,
    /// Log message content
    pub message: String,
    /// Optional additional metadata as JSON
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,
}

/// Thread-safe log storage with circular buffer behavior
#[derive(Clone)]
pub struct LogStore {
    /// Maximum number of logs to store
    max_capacity: usize,
    /// Current counter for log IDs
    counter: Arc<AtomicUsize>,
    /// Map of log ID to log entry
    logs: Arc<DashMap<usize, LogEntry>>,
}

impl LogStore {
    /// Create a new log store with the specified maximum capacity
    ///
    /// # Arguments
    ///
    /// * `max_capacity` - Maximum number of log entries to store
    ///
    /// # Example
    ///
    /// ```
    /// use ndnm_exdoida::storage::LogStore;
    ///
    /// let store = LogStore::new(10000);
    /// ```
    pub fn new(max_capacity: usize) -> Self {
        Self {
            max_capacity,
            counter: Arc::new(AtomicUsize::new(0)),
            logs: Arc::new(DashMap::new()),
        }
    }

    /// Add a new log entry to the store
    ///
    /// If the store is at capacity, the oldest entry will be removed.
    ///
    /// # Arguments
    ///
    /// * `level` - Log level (info, warn, error, debug, trace)
    /// * `source` - Source service name
    /// * `message` - Log message content
    /// * `metadata` - Optional additional metadata
    ///
    /// # Example
    ///
    /// ```
    /// use ndnm_exdoida::storage::LogStore;
    ///
    /// let store = LogStore::new(10000);
    /// store.add("info", "hermes", "Service started", None);
    /// ```
    pub fn add(
        &self,
        level: impl Into<String>,
        source: impl Into<String>,
        message: impl Into<String>,
        metadata: Option<serde_json::Value>,
    ) {
        let id = self.counter.fetch_add(1, Ordering::SeqCst);

        let entry = LogEntry {
            id,
            timestamp: Utc::now(),
            level: level.into(),
            source: source.into(),
            message: message.into(),
            metadata,
        };

        // Remove oldest entry if at capacity
        if self.logs.len() >= self.max_capacity {
            // Find the oldest entry (lowest ID)
            if let Some(oldest_id) = self.logs.iter().map(|entry| *entry.key()).min() {
                self.logs.remove(&oldest_id);
            }
        }

        self.logs.insert(id, entry);
    }

    /// Get the most recent log entries up to the specified limit
    ///
    /// Entries are returned in reverse chronological order (newest first).
    ///
    /// # Arguments
    ///
    /// * `limit` - Maximum number of entries to return
    ///
    /// # Returns
    ///
    /// Vector of log entries, newest first
    ///
    /// # Example
    ///
    /// ```
    /// use ndnm_exdoida::storage::LogStore;
    ///
    /// let store = LogStore::new(10000);
    /// store.add("info", "hermes", "Service started", None);
    /// let recent = store.get_recent(100);
    /// assert_eq!(recent.len(), 1);
    /// ```
    pub fn get_recent(&self, limit: usize) -> Vec<LogEntry> {
        let mut entries: Vec<LogEntry> = self
            .logs
            .iter()
            .map(|entry| entry.value().clone())
            .collect();

        // Sort by ID descending (newest first)
        entries.sort_by(|a, b| b.id.cmp(&a.id));

        // Take only the requested number
        entries.into_iter().take(limit).collect()
    }

    /// Get the total number of log entries currently stored
    ///
    /// # Returns
    ///
    /// Number of log entries in the store
    ///
    /// # Example
    ///
    /// ```
    /// use ndnm_exdoida::storage::LogStore;
    ///
    /// let store = LogStore::new(10000);
    /// assert_eq!(store.count(), 0);
    /// store.add("info", "hermes", "Test", None);
    /// assert_eq!(store.count(), 1);
    /// ```
    pub fn count(&self) -> usize {
        self.logs.len()
    }

    /// Clear all log entries from the store
    ///
    /// This removes all stored logs but does not reset the ID counter.
    ///
    /// # Example
    ///
    /// ```
    /// use ndnm_exdoida::storage::LogStore;
    ///
    /// let store = LogStore::new(10000);
    /// store.add("info", "hermes", "Test", None);
    /// assert_eq!(store.count(), 1);
    /// store.clear();
    /// assert_eq!(store.count(), 0);
    /// ```
    pub fn clear(&self) {
        self.logs.clear();
    }

    /// Get the maximum capacity of this store
    ///
    /// # Returns
    ///
    /// Maximum number of log entries that can be stored
    pub fn max_capacity(&self) -> usize {
        self.max_capacity
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_log_store_creation() {
        let store = LogStore::new(100);
        assert_eq!(store.count(), 0);
        assert_eq!(store.max_capacity(), 100);
    }

    #[test]
    fn test_add_and_retrieve_logs() {
        let store = LogStore::new(100);

        store.add("info", "hermes", "Test message 1", None);
        store.add("error", "brazil", "Test message 2", None);

        assert_eq!(store.count(), 2);

        let recent = store.get_recent(10);
        assert_eq!(recent.len(), 2);

        // Should be in reverse chronological order
        assert_eq!(recent[0].message, "Test message 2");
        assert_eq!(recent[1].message, "Test message 1");
    }

    #[test]
    fn test_circular_buffer_behavior() {
        let store = LogStore::new(3);

        store.add("info", "test", "Message 1", None);
        store.add("info", "test", "Message 2", None);
        store.add("info", "test", "Message 3", None);
        assert_eq!(store.count(), 3);

        // This should remove the oldest entry
        store.add("info", "test", "Message 4", None);
        assert_eq!(store.count(), 3);

        let recent = store.get_recent(10);
        assert_eq!(recent.len(), 3);

        // Message 1 should be gone
        assert!(!recent.iter().any(|e| e.message == "Message 1"));
        assert!(recent.iter().any(|e| e.message == "Message 4"));
    }

    #[test]
    fn test_get_recent_with_limit() {
        let store = LogStore::new(100);

        for i in 0..10 {
            store.add("info", "test", format!("Message {}", i), None);
        }

        assert_eq!(store.count(), 10);

        let recent = store.get_recent(5);
        assert_eq!(recent.len(), 5);

        // Should get the 5 most recent
        assert_eq!(recent[0].message, "Message 9");
        assert_eq!(recent[4].message, "Message 5");
    }

    #[test]
    fn test_clear_logs() {
        let store = LogStore::new(100);

        store.add("info", "test", "Message 1", None);
        store.add("info", "test", "Message 2", None);
        assert_eq!(store.count(), 2);

        store.clear();
        assert_eq!(store.count(), 0);

        let recent = store.get_recent(10);
        assert_eq!(recent.len(), 0);
    }

    #[test]
    fn test_log_entry_with_metadata() {
        let store = LogStore::new(100);

        let metadata = serde_json::json!({
            "request_id": "123",
            "duration_ms": 42
        });

        store.add("info", "hermes", "Request completed", Some(metadata.clone()));

        let recent = store.get_recent(1);
        assert_eq!(recent.len(), 1);
        assert_eq!(recent[0].metadata, Some(metadata));
    }
}
