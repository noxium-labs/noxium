use kafka::producer::{Producer, Record, RequiredAcks};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::time::Duration;
use std::env;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use log::{info, error, warn};
use signal_hook::{consts::TERM_SIGNALS, iterator::Signals};
use serde::{Serialize, Deserialize};
use std::process::exit;

// Struct for configuration settings
#[derive(Serialize, Deserialize, Debug)]
struct Config {
    kafka_broker: String,
    topic: String,
    input_file: String,
    ack_timeout_secs: u64,
    required_acks: i16,
}

// Default values for configuration
impl Default for Config {
    fn default() -> Self {
        Self {
            kafka_broker: String::from("127.0.0.1:9092"),
            topic: String::from("data_pipeline"),
            input_file: String::from("data/input.txt"),
            ack_timeout_secs: 1,
            required_acks: 1, // Corresponds to RequiredAcks::One
        }
    }
}

// Function to load configuration from environment variables
fn load_config() -> Config {
    let kafka_broker = env::var("KAFKA_BROKER").unwrap_or_else(|_| "127.0.0.1:9092".to_string());
    let topic = env::var("TOPIC").unwrap_or_else(|_| "data_pipeline".to_string());
    let input_file = env::var("INPUT_FILE").unwrap_or_else(|_| "data/input.txt".to_string());
    let ack_timeout_secs = env::var("ACK_TIMEOUT_SECS")
        .unwrap_or_else(|_| "1".to_string())
        .parse::<u64>()
        .unwrap_or(1);
    let required_acks = env::var("REQUIRED_ACKS")
        .unwrap_or_else(|_| "1".to_string())
        .parse::<i16>()
        .unwrap_or(1);

    Config {
        kafka_broker,
        topic,
        input_file,
        ack_timeout_secs,
        required_acks,
    }
}

// Main function
fn main() {
    env_logger::init(); // Initialize logger

    let config = load_config();
    info!("Loaded configuration: {:?}", config);

    let producer = Producer::from_hosts(vec![config.kafka_broker.clone()])
        .with_ack_timeout(Duration::from_secs(config.ack_timeout_secs))
        .with_required_acks(RequiredAcks::from(config.required_acks))
        .create()
        .unwrap_or_else(|e| {
            error!("Failed to create producer: {}", e);
            exit(1);
        });

    let file = File::open(&config.input_file).unwrap_or_else(|e| {
        error!("Failed to open input file: {}", e);
        exit(1);
    });

    let reader = BufReader::new(file);
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();

    let signals = Signals::new(TERM_SIGNALS).expect("Failed to create signal handler");
    thread::spawn(move || {
        for sig in signals.forever() {
            warn!("Received termination signal: {:?}", sig);
            r.store(false, Ordering::SeqCst);
        }
    });

    let mut producer = producer;

    for line in reader.lines() {
        if !running.load(Ordering::SeqCst) {
            warn!("Shutting down gracefully...");
            break;
        }

        match line {
            Ok(chunk) => {
                match producer.send(&Record::from_value(&config.topic, chunk.clone())) {
                    Ok(_) => info!("Sent: {}", chunk),
                    Err(e) => error!("Failed to send message: {}", e),
                }
            }
            Err(e) => error!("Failed to read line: {}", e),
        }

        // Simulate processing delay or to avoid tight loop in case of no data
        thread::sleep(Duration::from_millis(100));
    }

    info!("Producer has been stopped");
}