use std::net::TcpStream;
use std::io::{Write, Read};
use std::thread;
use std::time::Duration;
use serde_json::Value;
use log::{info, error, warn};
use std::env;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use signal_hook::{consts::TERM_SIGNALS, iterator::Signals};

// Struct for configuration settings
#[derive(Debug)]
struct Config {
    server_address: String,
    data_sources: Vec<String>,
    sleep_duration_secs: u64,
}

// Default values for configuration
impl Default for Config {
    fn default() -> Self {
        Self {
            server_address: String::from("127.0.0.1:5500"),
            data_sources: vec![
                r#"{"sensor_id": "temp_sensor_1", "value": 22.5}"#.to_string(),
                r#"{"sensor_id": "temp_sensor_2", "value": 23.0}"#.to_string(),
                r#"{"sensor_id": "humidity_sensor_1", "value": 45.0}"#.to_string(),
            ],
            sleep_duration_secs: 10,
        }
    }
}

// Function to load configuration from environment variables
fn load_config() -> Config {
    let server_address = env::var("SERVER_ADDRESS").unwrap_or_else(|_| "127.0.0.1:5500".to_string());
    let data_sources = env::var("DATA_SOURCES")
        .unwrap_or_else(|_| r#"["{\"sensor_id\": \"temp_sensor_1\", \"value\": 22.5}", "{\"sensor_id\": \"temp_sensor_2\", \"value\": 23.0}", "{\"sensor_id\": \"humidity_sensor_1\", \"value\": 45.0}"]"#.to_string())
        .split(',')
        .map(|s| s.trim().to_string())
        .collect();
    let sleep_duration_secs = env::var("SLEEP_DURATION_SECS")
        .unwrap_or_else(|_| "10".to_string())
        .parse::<u64>()
        .unwrap_or(10);

    Config {
        server_address,
        data_sources,
        sleep_duration_secs,
    }
}

// Function to send aggregated data to the server
fn send_aggregated_data(stream: &mut TcpStream, data: &str) {
    let message = format!("Aggregated Data: {}\n", data);
    if let Err(e) = stream.write_all(message.as_bytes()) {
        error!("Failed to send data: {}", e);
    }
}

// Main function
fn main() {
    env_logger::init(); // Initialize logger

    let config = load_config();
    info!("Loaded configuration: {:?}", config);

    let mut stream = TcpStream::connect(&config.server_address)
        .unwrap_or_else(|e| {
            error!("Could not connect to server: {}", e);
            std::process::exit(1);
        });

    let mut aggregated_data = vec![];
    for data in config.data_sources {
        match serde_json::from_str::<Value>(&data) {
            Ok(v) => aggregated_data.push(v),
            Err(e) => {
                warn!("Failed to parse data source '{}': {}", data, e);
                continue;
            }
        }
    }

    let aggregated_json = serde_json::to_string(&aggregated_data)
        .unwrap_or_else(|e| {
            error!("Failed to serialize aggregated data: {}", e);
            std::process::exit(1);
        });

    info!("Aggregated Data: {}", aggregated_json);
    send_aggregated_data(&mut stream, &aggregated_json);

    // Graceful shutdown handling
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();
    let signals = Signals::new(TERM_SIGNALS).expect("Failed to create signal handler");
    thread::spawn(move || {
        for sig in signals.forever() {
            warn!("Received termination signal: {:?}", sig);
            r.store(false, Ordering::SeqCst);
        }
    });

    // Main loop
    while running.load(Ordering::SeqCst) {
        thread::sleep(Duration::from_secs(config.sleep_duration_secs));
    }

    info!("Shutting down gracefully...");
}