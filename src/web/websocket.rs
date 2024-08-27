use futures::{StreamExt, SinkExt}; // For working with async streams and sinks
use std::collections::HashMap; // To store client data and mappings
use std::sync::{Arc, Mutex}; // For thread-safe shared state
use tokio::net::TcpListener; // To accept incoming TCP connections
use tokio_tungstenite::{accept_async, WebSocketStream}; // For WebSocket handling
use tungstenite::protocol::Message; // For WebSocket messages
use tokio::sync::broadcast; // For broadcasting messages to multiple clients
use log::{info, error, warn}; // For logging information, warnings, and errors

// Type aliases for managing client sender, receiver, and username mappings
type SenderMap = Arc<Mutex<HashMap<u32, tokio::sync::broadcast::Sender<String>>>>;
type ReceiverMap = Arc<Mutex<HashMap<u32, tokio::sync::broadcast::Receiver<String>>>>;
type UserMap = Arc<Mutex<HashMap<u32, String>>>;

#[tokio::main]
async fn main() {
    env_logger::init(); // Initialize logging

    let addr = "127.0.0.1:8080"; // Define the server address
    let listener = TcpListener::bind(addr).await.expect("Failed to bind"); // Bind the server to the address

    // Initialize shared state for managing client connections and usernames
    let sender_map = Arc::new(Mutex::new(HashMap::new()));
    let receiver_map = Arc::new(Mutex::new(HashMap::new()));
    let user_map = Arc::new(Mutex::new(HashMap::new()));

    // Create a broadcast channel for sending messages to all connected clients
    let (broadcast_tx, _) = broadcast::channel(100);

    info!("WebSocket server listening on {}", addr);

    let mut client_id = 0; // Counter for assigning unique client IDs

    // Main loop to accept incoming TCP connections
    while let Ok((stream, _)) = listener.accept().await {
        // Create a broadcast channel for each client
        let (tx, rx) = broadcast::channel(100);
        let mut tx = tx.clone();
        let mut rx = rx.clone();
        let id = client_id;
        client_id += 1; // Increment client ID for the next connection

        // Clone Arc pointers for shared access across tasks
        let sender_map = Arc::clone(&sender_map);
        let receiver_map = Arc::clone(&receiver_map);
        let user_map = Arc::clone(&user_map);

        // Spawn a new task to handle the client connection
        tokio::spawn(async move {
            // Upgrade the TCP stream to a WebSocket stream
            let ws_stream = accept_async(stream)
                .await
                .expect("Error during WebSocket handshake");

            let (mut ws_sender, mut ws_receiver) = ws_stream.split(); // Split the WebSocket stream into sender and receiver

            // Store the client's sender and receiver in shared maps
            {
                let mut sender_map = sender_map.lock().unwrap();
                sender_map.insert(id, tx);
            }

            {
                let mut receiver_map = receiver_map.lock().unwrap();
                receiver_map.insert(id, rx);
            }

            // Set a default username for the client
            {
                let mut user_map = user_map.lock().unwrap();
                user_map.insert(id, format!("User{}", id));
            }

            info!("Client {} connected", id); // Log the new connection

            // Handle incoming messages from the client
            while let Some(message) = ws_receiver.next().await {
                match message {
                    Ok(Message::Text(text)) => {
                        // Process text messages from the client
                        if text.starts_with("/nick ") {
                            // Command to change the client's username
                            let new_username = text.trim_start_matches("/nick ").trim().to_string();
                            let mut user_map = user_map.lock().unwrap();
                            if new_username.is_empty() {
                                ws_sender.send(Message::Text("Username cannot be empty".to_string())).await.expect("Failed to send message");
                            } else {
                                let old_username = user_map.insert(id, new_username.clone());
                                let message = format!("{} changed username to {}", old_username.unwrap_or("Unknown".to_string()), new_username);
                                broadcast_message(&sender_map, &message).await;
                            }
                        } else if text.starts_with("/msg ") {
                            // Command to send a private message to another user
                            let parts: Vec<&str> = text.splitn(3, ' ').collect();
                            if parts.len() < 3 {
                                ws_sender.send(Message::Text("Usage: /msg <user> <message>".to_string())).await.expect("Failed to send message");
                                continue;
                            }
                            let recipient_username = parts[1];
                            let message = parts[2];
                            let recipient_id = {
                                let user_map = user_map.lock().unwrap();
                                user_map.iter().find_map(|(&id, username)| if username == recipient_username { Some(id) } else { None })
                            };
                            if let Some(recipient_id) = recipient_id {
                                let sender_map = sender_map.lock().unwrap();
                                if let Some(tx) = sender_map.get(&recipient_id) {
                                    tx.send(format!("Private message from {}: {}", user_map.lock().unwrap().get(&id).unwrap_or(&"Unknown".to_string()), message)).expect("Failed to send private message");
                                }
                            } else {
                                ws_sender.send(Message::Text(format!("User {} not found", recipient_username))).await.expect("Failed to send message");
                            }
                        } else {
                            // Broadcast the message to all connected clients
                            let message = format!("{}: {}", user_map.lock().unwrap().get(&id).unwrap_or(&"Unknown".to_string()), text);
                            broadcast_message(&sender_map, &message).await;
                        }
                    }
                    Ok(Message::Close(_)) => {
                        info!("Client {} disconnected", id); // Log client disconnection
                        break; // Exit the loop on client disconnection
                    }
                    Err(e) => {
                        error!("Error from client {}: {}", id, e); // Log errors
                        break; // Exit the loop on error
                    }
                    _ => (), // Ignore other types of messages
                }
            }

            // Clean up client state upon disconnection
            {
                let mut sender_map = sender_map.lock().unwrap();
                sender_map.remove(&id);
            }

            {
                let mut receiver_map = receiver_map.lock().unwrap();
                receiver_map.remove(&id);
            }

            {
                let mut user_map = user_map.lock().unwrap();
                user_map.remove(&id);
            }
        });
    }
}

// Function to broadcast a message to all connected clients
async fn broadcast_message(sender_map: &SenderMap, message: &str) {
    let sender_map = sender_map.lock().unwrap();
    for (_, tx) in sender_map.iter() {
        tx.send(message.to_string()).expect("Failed to broadcast message");
    }
}