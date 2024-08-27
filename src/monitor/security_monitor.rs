use std::net::TcpStream;
use std::io::{Write, BufRead, BufReader, Error as IoError};
use std::thread;
use std::time::Duration;
use std::fs::File;
use log::{info, error};
use std::env;
use signal_hook::{consts::TERM_SIGNALS, iterator::Signals};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

// Struct for configuration settings
#[derive(Debug)]
struct Config {
    server_address: String,
    event_file_path: String,
    sleep_duration_secs: u64,
}

// Default values for configuration
impl Default for Config {
    fn default() -> Self {
        Self {
            server_address: String::from("127.0.0.1:5500"),
            event_file_path: String::from("events.txt"),
            sleep_duration_secs: 5,
        }
    }
}

// Function to load configuration from environment variables
fn load_config() -> Config {
    let server_address = env::var("SERVER_ADDRESS").unwrap_or_else(|_| "127.0.0.1:5500".to_string());
    let event_file_path = env::var("EVENT_FILE_PATH").unwrap_or_else(|_| "events.txt".to_string());
    let sleep_duration_secs = env::var("SLEEP_DURATION_SECS")
        .unwrap_or_else(|_| "5".to_string())
        .parse::<u64>()
        .unwrap_or(5);

    Config {
        server_address,
        event_file_path,
        sleep_duration_secs,
    }
}

// Function to log security events
fn log_security_event(stream: &mut TcpStream, event: &str) -> Result<(), IoError> {
    let message = format!("Security Event: {}\n", event);
    stream.write_all(message.as_bytes())?;
    Ok(())
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

    let file = File::open(&config.event_file_path)
        .unwrap_or_else(|e| {
            error!("Could not open event file: {}", e);
            std::process::exit(1);
        });

    let reader = BufReader::new(file);
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();
    let signals = Signals::new(TERM_SIGNALS).expect("Failed to create signal handler");
    thread::spawn(move || {
        for sig in signals.forever() {
            error!("Received termination signal: {:?}", sig);
            r.store(false, Ordering::SeqCst);
        }
    });

    for line in reader.lines() {
        if !running.load(Ordering::SeqCst) {
            info!("Shutting down gracefully...");
            break;
        }

        match line {
            Ok(event) => {
                println!("Security Event: {}", event);
                if let Err(err) = log_security_event(&mut stream, &event) {
                    error!("Failed to log security event: {}", err);
                }
            }
            Err(err) => {
                error!("Failed to read event from file: {}", err);
            }
        }

        thread::sleep(Duration::from_secs(config.sleep_duration_secs));
    }

    info!("Completed processing all events or stopped due to shutdown.");
}