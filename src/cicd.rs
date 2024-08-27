use std::env;
use std::process::Command;
use std::time::{Duration, SystemTime};
use std::thread::sleep;
use std::str::FromStr;

#[derive(Debug)]
enum Action {
    Run { command: String },
    Schedule { command: String, delay_seconds: u64 },
}

impl Action {
    fn from_args(args: &[String]) -> Result<Action, String> {
        if args.len() < 2 {
            return Err("Usage: <command> [--schedule <seconds_from_now>]".to_string());
        }
        
        let command = args[1].clone();

        if args.len() == 4 && args[2] == "--schedule" {
            let delay_seconds = u64::from_str(&args[3])
                .map_err(|_| "Invalid delay format. Must be an integer.".to_string())?;
            Ok(Action::Schedule { command, delay_seconds })
        } else {
            Ok(Action::Run { command })
        }
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    
    match Action::from_args(&args) {
        Ok(action) => {
            match action {
                Action::Run { command } => {
                    if let Err(e) = execute_command(&command) {
                        eprintln!("Command execution failed: {}", e);
                        std::process::exit(1);
                    }
                }
                Action::Schedule { command, delay_seconds } => {
                    let delay = Duration::new(delay_seconds, 0);
                    println!("Scheduling '{}' to run in {} seconds", command, delay_seconds);
                    sleep(delay);
                    
                    if let Err(e) = execute_command(&command) {
                        eprintln!("Command execution failed: {}", e);
                        std::process::exit(1);
                    }
                }
            }
        }
        Err(e) => {
            eprintln!("{}", e);
            std::process::exit(1);
        }
    }
}

fn execute_command(command: &str) -> Result<(), String> {
    let status = Command::new("sh")
        .arg("-c")
        .arg(command)
        .status()
        .map_err(|e| e.to_string())?;

    if status.success() {
        println!("Command executed successfully");
        Ok(())
    } else {
        Err(format!("Command failed with status: {}", status))
    }
}