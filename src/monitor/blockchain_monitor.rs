use serde::{Serialize, Deserialize};
use std::net::{TcpStream, TcpListener};
use std::io::{Write, Read};
use std::time::{SystemTime, UNIX_EPOCH};
use std::thread;
use md5;
use std::sync::{Arc, Mutex};

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Block {
    index: u64,
    timestamp: u128,
    data: String,
    prev_hash: String,
    hash: String,
}

impl Block {
    fn new(index: u64, data: String, prev_hash: String) -> Block {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_millis();
        let hash = format!("{:x}", md5::compute(format!("{}{}{}{}", index, timestamp, &data, &prev_hash)));
        Block {
            index,
            timestamp,
            data,
            prev_hash,
            hash,
        }
    }
}

fn validate_blockchain(blockchain: &[Block]) -> bool {
    for i in 1..blockchain.len() {
        let current = &blockchain[i];
        let previous = &blockchain[i - 1];

        if current.prev_hash != previous.hash {
            return false;
        }

        let expected_hash = format!("{:x}", md5::compute(format!("{}{}{}{}", current.index, current.timestamp, &current.data, &current.prev_hash)));
        if current.hash != expected_hash {
            return false;
        }
    }
    true
}

fn handle_client(stream: TcpStream, blockchain: Arc<Mutex<Vec<Block>>>) {
    let mut stream = stream;
    let mut buffer = [0; 1024];

    loop {
        let size = match stream.read(&mut buffer) {
            Ok(size) if size > 0 => size,
            _ => break,
        };

        let message = String::from_utf8_lossy(&buffer[..size]);
        let block: Block = match serde_json::from_str(&message) {
            Ok(block) => block,
            Err(_) => continue,
        };

        let mut blockchain = blockchain.lock().unwrap();
        blockchain.push(block.clone());

        if !validate_blockchain(&blockchain) {
            eprintln!("Blockchain validation failed!");
        }
    }
}

fn start_server() {
    let listener = TcpListener::bind("127.0.0.1:5500").expect("Could not bind to address");
    let blockchain = Arc::new(Mutex::new(vec![]));

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let blockchain = Arc::clone(&blockchain);
                thread::spawn(move || handle_client(stream, blockchain));
            }
            Err(e) => eprintln!("Failed to accept connection: {}", e),
        }
    }
}

fn main() {
    let server_thread = thread::spawn(|| start_server());

    thread::sleep(std::time::Duration::from_secs(1)); // Give server a moment to start

    let mut stream = TcpStream::connect("127.0.0.1:5500").expect("Could not connect to server");
    let mut blockchain = vec![];

    for i in 0..10 {
        let prev_hash = if blockchain.is_empty() {
            String::from("0")
        } else {
            blockchain.last().unwrap().hash.clone()
        };
        let block = Block::new(i, format!("Block {}", i), prev_hash);
        blockchain.push(block.clone());

        let message = serde_json::to_string(&block).unwrap();
        stream.write_all(message.as_bytes()).unwrap();
        thread::sleep(std::time::Duration::from_secs(1));
    }

    server_thread.join().expect("Server thread panicked");
}