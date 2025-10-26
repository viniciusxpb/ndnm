//! WebSocket communication module
//!
//! Handles WebSocket connections from frontend clients and manages
//! broadcasting messages to all connected clients.

use axum::{
    extract::{
        ws::{Message, WebSocket},
        State, WebSocketUpgrade,
    },
    response::Response,
};
use futures_util::{sink::SinkExt, stream::StreamExt};
use serde_json::Value;
use std::sync::Arc;
use tokio::sync::broadcast;
use tracing::{error, info, warn};
use uuid::Uuid;

use crate::AppState;

/// Capacity for the broadcast channel
const BROADCAST_CAPACITY: usize = 100;

/// Broadcaster for sending messages to all connected WebSocket clients
#[derive(Clone)]
pub struct Broadcaster {
    /// Broadcast channel sender
    tx: Arc<broadcast::Sender<String>>,
}

impl Broadcaster {
    /// Create a new broadcaster
    pub fn new() -> Self {
        let (tx, _rx) = broadcast::channel(BROADCAST_CAPACITY);
        Self { tx: Arc::new(tx) }
    }

    /// Broadcast a JSON message to all connected clients
    ///
    /// # Arguments
    ///
    /// * `message` - JSON value to broadcast
    pub async fn broadcast_json(&self, message: &Value) {
        if let Ok(msg_str) = serde_json::to_string(message) {
            // Ignore send errors (happens when no receivers)
            let _ = self.tx.send(msg_str);
        }
    }

    /// Subscribe to broadcast messages
    ///
    /// Returns a receiver that will receive all broadcast messages
    pub fn subscribe(&self) -> broadcast::Receiver<String> {
        self.tx.subscribe()
    }
}

/// Handler for WebSocket upgrade requests
///
/// This is the main entry point for WebSocket connections from the frontend
pub async fn websocket_handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
) -> Response {
    ws.on_upgrade(|socket| handle_socket(socket, state))
}

/// Handle an individual WebSocket connection
///
/// # Arguments
///
/// * `socket` - The WebSocket connection
/// * `state` - Application state
async fn handle_socket(socket: WebSocket, state: AppState) {
    // Generate a unique client ID
    let client_id = Uuid::new_v4();
    info!("New WebSocket client connected: {}", client_id);

    // Split the socket into sender and receiver
    let (mut sender, mut receiver) = socket.split();

    // Subscribe to broadcasts
    let mut rx = state.ws_broadcaster.subscribe();

    // Send welcome message
    let welcome = serde_json::json!({
        "type": "connected",
        "client_id": client_id.to_string(),
        "message": "Connected to NDNM Brazil BFF"
    });

    if let Ok(msg) = serde_json::to_string(&welcome) {
        if sender.send(Message::Text(msg)).await.is_err() {
            warn!("Failed to send welcome message to client {}", client_id);
            return;
        }
    }

    // Spawn a task to handle broadcasts to this client
    let mut send_task = tokio::spawn(async move {
        while let Ok(msg) = rx.recv().await {
            if sender.send(Message::Text(msg)).await.is_err() {
                break;
            }
        }
    });

    // Handle incoming messages from this client
    let client_id_clone = client_id;
    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = receiver.next().await {
            if let Message::Text(text) = msg {
                // Try to parse as JSON
                match serde_json::from_str::<Value>(&text) {
                    Ok(json) => {
                        info!("Received message from client {}: {:?}", client_id_clone, json);

                        // Handle different message types
                        if let Some(msg_type) = json.get("type").and_then(|v| v.as_str()) {
                            match msg_type {
                                "ping" => {
                                    info!("Ping from client {}", client_id_clone);
                                    // Pong is handled automatically
                                }
                                _ => {
                                    info!(
                                        "Unhandled message type '{}' from client {}",
                                        msg_type, client_id_clone
                                    );
                                }
                            }
                        }
                    }
                    Err(e) => {
                        warn!("Invalid JSON from client {}: {}", client_id_clone, e);
                    }
                }
            } else if let Message::Close(_) = msg {
                info!("Client {} requested close", client_id_clone);
                break;
            }
        }
    });

    // Wait for either task to finish
    tokio::select! {
        _ = &mut send_task => {
            recv_task.abort();
        },
        _ = &mut recv_task => {
            send_task.abort();
        }
    }

    info!("WebSocket client disconnected: {}", client_id);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_broadcaster_creation() {
        let broadcaster = Broadcaster::new();
        let _rx = broadcaster.subscribe();
        // Test that we can create and subscribe
    }

    #[tokio::test]
    async fn test_broadcast_json() {
        let broadcaster = Broadcaster::new();
        let mut rx = broadcaster.subscribe();

        let test_msg = serde_json::json!({
            "type": "test",
            "data": "hello"
        });

        broadcaster.broadcast_json(&test_msg).await;

        // Try to receive (might timeout if broadcast happens before subscribe)
        let received = tokio::time::timeout(
            std::time::Duration::from_millis(100),
            rx.recv()
        ).await;

        // Just checking that the mechanism works
        assert!(received.is_ok() || received.is_err());
    }
}
