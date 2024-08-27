use sysinfo::{ProcessorExt, System, SystemExt};
use std::net::TcpStream;
use std::io::{Write, Error as IoError};
use std::thread;
use std::time::Duration;
use log::{info, error, warn};
use std::env;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use signal_hook::{consts::TERM_SIGNALS, iterator::Signals};

// Struct for configuration settings
#[derive(Debug)]
struct Config {
    server_address: String,
    refresh_interval_secs: u64,
}

// Default values for configuration
impl Default for Config {
    fn default() -> Self {
        Self {
            server_address: String::from("127.0.0.1:5500"),
            refresh_interval_secs: 1,
        }
    }
}

// Function to load configuration from environment variables
fn load_config() -> Config {
    let server_address = env::var("SERVER_ADDRESS").unwrap_or_else(|_| "127.0.0.1:5500".to_string());
    let refresh_interval_secs = env::var("REFRESH_INTERVAL_SECS")
        .unwrap_or_else(|_| "1".to_string())
        .parse::<u64>()
        .unwrap_or(1);

    Config {
        server_address,
        refresh_interval_secs,
    }
}

// Function to log CPU usage
fn log_cpu_usage(stream: &mut TcpStream, usage: f32) -> Result<(), IoError> {
    let message = format!("CPU Usage: {:.2}%\n", usage);
    stream.write_all(message.as_bytes())?;
    Ok(())
}

// Main function
fn main() {
    env_logger::init(); // Initialize logger

    let config = load_config();
    info!("Loaded configuration: {:?}", config);

    let mut system = System::new_all();
    let mut stream = TcpStream::connect(&config.server_address)
        .unwrap_or_else(|e| {
            error!("Could not connect to server: {}", e);
            std::process::exit(1);
        });

    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();
    let signals = Signals::new(TERM_SIGNALS).expect("Failed to create signal handler");
    thread::spawn(move || {
        for sig in signals.forever() {
            warn!("Received termination signal: {:?}", sig);
            r.store(false, Ordering::SeqCst);
        }
    });

    while running.load(Ordering::SeqCst) {
        if let Err(err) = system.refresh_all() {
            error!("Failed to refresh system data: {}", err);
        } else if let Some(cpu_usage) = system.global_processor_info().cpu_usage() {
            info!("CPU Usage: {:.2}%", cpu_usage);
            if let Err(err) = log_cpu_usage(&mut stream, cpu_usage) {
                error!("Failed to log CPU usage: {}", err);
            }
        } else {
            error!("Failed to retrieve CPU usage");
        }

        thread::sleep(Duration::from_secs(config.refresh_interval_secs));
    }

    info!("Shutting down gracefully...");
}