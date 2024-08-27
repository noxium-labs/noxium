use kafka::consumer::{Consumer, FetchOffset, GroupOffsetStorage};
use std::time::{Duration, Instant};
use std::fs::{OpenOptions, File};
use std::io::{Write, BufWriter};
use log::{info, error, warn};
use serde::{Serialize, Deserialize};
use std::env;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::process::exit;
use signal_hook::{consts::TERM_SIGNALS, iterator::Signals};

// Struct for configuration settings
#[derive(Serialize, Deserialize, Debug)]
struct Config {
    kafka_broker: String,
    topic: String,
    group_id: String,
    output_file: String,
    polling_interval_secs: u64,
}

// Default values for configuration
impl Default for Config {
    fn default() -> Self {
        Self {
            kafka_broker: String::from(DEFAULT_KAFKA_BROKER),
            topic: String::from(DEFAULT_TOPIC),
            group_id: String::from(DEFAULT_GROUP_ID),
            output_file: String::from("data/output.txt"),
            polling_interval_secs: 1,
        }
    }
}

// Function to load configuration from environment variables
fn load_config() -> Config {
    let kafka_broker = env::var("KAFKA_BROKER").unwrap_or_else(|_| DEFAULT_KAFKA_BROKER.to_string());
    let topic = env::var("TOPIC").unwrap_or_else(|_| DEFAULT_TOPIC.to_string());
    let group_id = env::var("GROUP_ID").unwrap_or_else(|_| DEFAULT_GROUP_ID.to_string());
    let output_file = env::var("OUTPUT_FILE").unwrap_or_else(|_| "data/output.txt".to_string());
    let polling_interval_secs = env::var("POLLING_INTERVAL_SECS")
        .unwrap_or_else(|_| "1".to_string())
        .parse::<u64>()
        .unwrap_or(1);

    Config {
        kafka_broker,
        topic,
        group_id,
        output_file,
        polling_interval_secs,
    }
}

// Main function
fn main() {
    env_logger::init(); // Initialize logger

    let config = load_config();
    info!("Loaded configuration: {:?}", config);

    let consumer = Consumer::from_hosts(vec![config.kafka_broker.clone()])
        .with_topic(config.topic.clone())
        .with_group(config.group_id.clone())
        .with_fallback_offset(FetchOffset::Earliest)
        .with_offset_storage(GroupOffsetStorage::Kafka)
        .create()
        .unwrap_or_else(|e| {
            error!("Failed to create consumer: {}", e);
            exit(1);
        });

    let file = OpenOptions::new().create(true).append(true).open(&config.output_file);
    let mut writer = BufWriter::new(file.unwrap_or_else(|e| {
        error!("Failed to open output file: {}", e);
        exit(1);
    }));

    // Graceful shutdown handling
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();
    let signals = Signals::new(TERM_SIGNALS).expect("Failed to create signal handler");

    std::thread::spawn(move || {
        for sig in signals.forever() {
            warn!("Received termination signal: {:?}", sig);
            r.store(false, Ordering::SeqCst);
        }
    });

    let mut consumer = consumer;
    let polling_interval = Duration::from_secs(config.polling_interval_secs);

    // Main polling loop
    while running.load(Ordering::SeqCst) {
        match consumer.poll() {
            Ok(message_sets) => {
                for ms in message_sets.iter() {
                    for m in ms.messages() {
                        if let Ok(chunk) = String::from_utf8(m.value.to_vec()) {
                            info!("Received: {}", chunk);
                            if let Err(e) = writeln!(writer, "{}", chunk) {
                                error!("Failed to write to file: {}", e);
                            }
                        } else {
                            warn!("Failed to parse message as UTF-8");
                        }
                    }
                    if let Err(e) = consumer.consume_messageset(ms) {
                        error!("Failed to consume message set: {}", e);
                    }
                }
                if let Err(e) = consumer.commit_consumed() {
                    error!("Failed to commit consumed messages: {}", e);
                }
            }
            Err(e) => error!("Error polling messages: {}", e),
        }

        std::thread::sleep(polling_interval);
    }

    info!("Shutting down gracefully");
}