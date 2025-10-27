//! # UDP Log Server
//!
//! Fire-and-forget UDP server for receiving logs from NDNM services.
//! Services can send logs without waiting for a response, ensuring
//! that the observability system never blocks the main application.

use crate::storage::LogStore;
use serde::Deserialize;
use std::sync::Arc;
use tokio::net::UdpSocket;
use tokio::time::Duration;
use chrono::Utc;
use tracing::{debug, error, warn};

/// Maximum size of a UDP datagram we'll accept (64KB)
const MAX_DATAGRAM_SIZE: usize = 65507;

/// Log message format expected from services
#[derive(Debug, Deserialize)]
struct UdpLogMessage {
    /// Log level (info, warn, error, debug, trace)
    level: String,
    /// Source service name
    source: String,
    /// Log message content
    message: String,
    /// Optional additional metadata
    #[serde(default)]
    metadata: Option<serde_json::Value>,
}

/// Start the UDP server to receive log messages
///
/// This server runs indefinitely, receiving UDP datagrams and storing them
/// in the provided LogStore. The fire-and-forget nature means clients never
/// wait for a response, ensuring minimal impact on application performance.
///
/// # Arguments
///
/// * `port` - UDP port to bind to
/// * `log_store` - Shared log store for persisting received logs
///
/// # Expected Message Format
///
/// Services should send JSON-formatted UDP datagrams:
///
/// ```json
/// {
///   "level": "info",
///   "source": "ndnm-hermes",
///   "message": "Graph execution completed",
///   "metadata": {
///     "graph_id": "123",
///     "duration_ms": 42
///   }
/// }
/// ```
///
/// # Example
///
/// ```no_run
/// use ndnm_exdoida::{storage::LogStore, udp_server};
/// use std::sync::Arc;
///
/// #[tokio::main]
/// async fn main() {
///     let log_store = Arc::new(LogStore::new(10000));
///     udp_server::start_udp_server(9514, log_store).await.unwrap();
/// }
/// ```
fn color_for_level(level: &str) -> &str {
    match level.to_lowercase().as_str() {
        "info" => "\x1b[32m",
        "warn" => "\x1b[33m",
        "error" => "\x1b[31m",
        "debug" => "\x1b[35m",
        _ => "\x1b[37m",
    }
}
fn print_log(level: &str, source: &str, message: &str) {
    let ts = Utc::now().to_rfc3339();
    let reset = "\x1b[0m";
    let ts_color = "\x1b[36m";
    let src_color = "\x1b[34m";
    let lvl_color = color_for_level(level);
    println!(
        "{ts_color}[{ts}]{reset} {src_color}[{source}]{reset} {lvl_color}{level}{reset}: {message}",
        ts_color = ts_color,
        ts = ts,
        reset = reset,
        src_color = src_color,
        source = source,
        lvl_color = lvl_color,
        level = level,
        message = message,
    );
}
pub async fn start_udp_server(
    port: u16,
    log_store: Arc<LogStore>,
) -> anyhow::Result<()> {
    let addr = format!("0.0.0.0:{}", port);
    let socket = UdpSocket::bind(&addr).await?;

    debug!("UDP log server listening on {}", addr);

    let mut buf = vec![0u8; MAX_DATAGRAM_SIZE];

    loop {
        match socket.recv_from(&mut buf).await {
            Ok((len, sender)) => {
                let data = &buf[..len];

                // Try to parse the JSON log message
                match serde_json::from_slice::<UdpLogMessage>(data) {
                    Ok(log_msg) => {
                        debug!(
                            "Received log from {} ({}): {} - {}",
                            sender, log_msg.source, log_msg.level, log_msg.message
                        );

                        // Store the log
                        log_store.add(
                            log_msg.level.clone(),
                            log_msg.source.clone(),
                            log_msg.message.clone(),
                            log_msg.metadata,
                        );
                        // Print colored log to console
                        print_log(&log_msg.level, &log_msg.source, &log_msg.message);
                    }
                    Err(e) => {
                        // Log parsing errors but don't crash
                        warn!(
                            "Failed to parse log message from {}: {} - Raw data: {}",
                            sender,
                            e,
                            String::from_utf8_lossy(data)
                        );

                        // Store as unparsed message
                        log_store.add(
                            "unknown",
                            format!("unparsed-{}", sender),
                            format!("Failed to parse: {}", String::from_utf8_lossy(data)),
                            None,
                        );
                        print_log(
                            "unknown",
                            &format!("unparsed-{}", sender),
                            &format!("Failed to parse: {}", String::from_utf8_lossy(data)),
                        );
                    }
                }
            }
            Err(e) => {
                error!("Error receiving UDP datagram: {}", e);
                // Continue running even on errors
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::net::UdpSocket;

    #[tokio::test]
    async fn test_udp_server_receives_logs() {
        let log_store = Arc::new(LogStore::new(100));
        let log_store_clone = log_store.clone();

        // Start server on a random port
        let server_socket = UdpSocket::bind("127.0.0.1:0").await.unwrap();
        let server_addr = server_socket.local_addr().unwrap();

        // Spawn server task
        tokio::spawn(async move {
            let mut buf = vec![0u8; MAX_DATAGRAM_SIZE];
            if let Ok((len, _sender)) = server_socket.recv_from(&mut buf).await {
                let data = &buf[..len];
                if let Ok(log_msg) = serde_json::from_slice::<UdpLogMessage>(data) {
                    log_store_clone.add(
                        log_msg.level,
                        log_msg.source,
                        log_msg.message,
                        log_msg.metadata,
                    );
                }
            }
        });

        // Give the server a moment to start
        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

        // Send a test log message
        let client_socket = UdpSocket::bind("127.0.0.1:0").await.unwrap();
        let test_message = serde_json::json!({
            "level": "info",
            "source": "test-client",
            "message": "Test log message"
        });

        let message_bytes = serde_json::to_vec(&test_message).unwrap();
        client_socket.send_to(&message_bytes, server_addr).await.unwrap();

        // Wait for message to be processed
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        // Verify log was stored
        assert_eq!(log_store.count(), 1);
        let logs = log_store.get_recent(1);
        assert_eq!(logs[0].level, "info");
        assert_eq!(logs[0].source, "test-client");
        assert_eq!(logs[0].message, "Test log message");
    }

    #[tokio::test]
    async fn test_udp_log_message_with_metadata() {
        let log_store = Arc::new(LogStore::new(100));
        let log_store_clone = log_store.clone();

        let server_socket = UdpSocket::bind("127.0.0.1:0").await.unwrap();
        let server_addr = server_socket.local_addr().unwrap();

        tokio::spawn(async move {
            let mut buf = vec![0u8; MAX_DATAGRAM_SIZE];
            if let Ok((len, _sender)) = server_socket.recv_from(&mut buf).await {
                let data = &buf[..len];
                if let Ok(log_msg) = serde_json::from_slice::<UdpLogMessage>(data) {
                    log_store_clone.add(
                        log_msg.level,
                        log_msg.source,
                        log_msg.message,
                        log_msg.metadata,
                    );
                }
            }
        });

        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

        let client_socket = UdpSocket::bind("127.0.0.1:0").await.unwrap();
        let test_message = serde_json::json!({
            "level": "error",
            "source": "hermes",
            "message": "Graph execution failed",
            "metadata": {
                "graph_id": "abc123",
                "error_code": 500
            }
        });

        let message_bytes = serde_json::to_vec(&test_message).unwrap();
        client_socket.send_to(&message_bytes, server_addr).await.unwrap();

        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        assert_eq!(log_store.count(), 1);
        let logs = log_store.get_recent(1);
        assert_eq!(logs[0].level, "error");
        assert!(logs[0].metadata.is_some());
    }

    #[test]
    fn test_deserialize_udp_log_message() {
        let json = r#"{
            "level": "warn",
            "source": "brazil",
            "message": "Connection timeout"
        }"#;

        let log_msg: UdpLogMessage = serde_json::from_str(json).unwrap();
        assert_eq!(log_msg.level, "warn");
        assert_eq!(log_msg.source, "brazil");
        assert_eq!(log_msg.message, "Connection timeout");
        assert!(log_msg.metadata.is_none());
    }

    #[test]
    fn test_deserialize_udp_log_message_with_metadata() {
        let json = r#"{
            "level": "info",
            "source": "hermes",
            "message": "Request processed",
            "metadata": {
                "request_id": "123",
                "duration_ms": 42
            }
        }"#;

        let log_msg: UdpLogMessage = serde_json::from_str(json).unwrap();
        assert_eq!(log_msg.level, "info");
        assert!(log_msg.metadata.is_some());
    }
}
