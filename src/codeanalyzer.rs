use std::process::Command;
use std::fs;
use serde_json::Value;
use reqwest::Client;
use serde::Deserialize;
use std::path::Path;
use tokio::fs as async_fs;
use std::error::Error;
use log::{info, error};
use clap::{Arg, Command as ClapCommand};

// Define a struct to represent the security report
#[derive(Deserialize)]
struct SecurityReport {
    vulnerabilities: Vec<String>,
    file_path: String,
    analysis_time: String,
}

// Configuration struct
struct Config {
    tool_path: String,
    vulnerability_db_url: String,
    file_paths: Vec<String>,
}

// Function to fetch the vulnerability database from a remote URL
async fn fetch_vulnerability_db(url: &str) -> Result<Value, reqwest::Error> {
    let client = Client::new();
    let res = client.get(url).send().await?;
    res.json().await
}

// Function to execute the security analysis tool on a specified file
fn run_analysis_tool(tool_path: &str, file_path: &str) -> Result<String, std::io::Error> {
    let output = Command::new(tool_path)
        .arg(file_path)
        .output()?;
    
    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        Err(std::io::Error::new(std::io::ErrorKind::Other, "Analysis failed"))
    }
}

// Function to analyze the security report and print vulnerabilities
fn analyze_report(report: &str) -> Result<(), serde_json::Error> {
    let report: SecurityReport = serde_json::from_str(report)?;
    
    println!("Analysis Report for File: {}", report.file_path);
    println!("Analysis Time: {}", report.analysis_time);
    
    for vulnerability in report.vulnerabilities.iter() {
        println!("Vulnerability found: {}", vulnerability);
    }
    
    Ok(())
}

// Function to save the analysis report to a file
async fn save_report_to_file(report: &str, file_path: &str) -> Result<(), Box<dyn Error>> {
    async_fs::write(file_path, report).await.map_err(|e| Box::new(e) as Box<dyn Error>)
}

// Function to compare fetched vulnerabilities with the local report
fn compare_vulnerabilities(local_report: &str, fetched_db: &Value) -> Vec<String> {
    let mut detected_vulnerabilities = Vec::new();
    let local_report: SecurityReport = serde_json::from_str(local_report).unwrap();
    
    for vulnerability in local_report.vulnerabilities.iter() {
        if fetched_db["vulnerabilities"].as_array().unwrap_or(&vec![]).contains(&Value::String(vulnerability.clone())) {
            detected_vulnerabilities.push(vulnerability.clone());
        }
    }
    
    detected_vulnerabilities
}

// Function to validate if the file path exists and is readable
fn validate_file_path(file_path: &str) -> Result<(), Box<dyn Error>> {
    let path = Path::new(file_path);
    
    if path.exists() && path.is_file() {
        Ok(())
    } else {
        Err(Box::new(std::io::Error::new(std::io::ErrorKind::NotFound, "File not found or is not a valid file")))
    }
}

// Function to analyze multiple files concurrently
async fn analyze_files(file_paths: Vec<String>, config: &Config) -> Result<(), Box<dyn Error>> {
    let mut handles = Vec::new();
    
    for file_path in file_paths {
        let tool_path = config.tool_path.clone();
        let report_file_path = format!("{}.report.json", file_path);
        
        handles.push(tokio::spawn(async move {
            match validate_file_path(&file_path) {
                Ok(()) => {
                    match run_analysis_tool(&tool_path, &file_path) {
                        Ok(analysis_report) => {
                            if let Err(e) = save_report_to_file(&analysis_report, &report_file_path).await {
                                error!("Failed to save report for {}: {}", file_path, e);
                            }
                            if let Err(e) = analyze_report(&analysis_report) {
                                error!("Failed to analyze report for {}: {}", file_path, e);
                            }
                        },
                        Err(e) => error!("Analysis failed for {}: {}", file_path, e),
                    }
                },
                Err(e) => error!("Validation failed for {}: {}", file_path, e),
            }
        }));
    }
    
    for handle in handles {
        handle.await.unwrap();
    }
    
    Ok(())
}

// Function to provide a detailed report summary
fn print_summary(file_paths: &[String], fetched_db: &Value) -> Result<(), Box<dyn Error>> {
    for file_path in file_paths {
        let report_file_path = format!("{}.report.json", file_path);
        let report_content = fs::read_to_string(&report_file_path)?;
        
        let detected_vulnerabilities = compare_vulnerabilities(&report_content, fetched_db);
        println!("Summary for File: {}", file_path);
        println!("Detected Vulnerabilities: {}", detected_vulnerabilities.len());
        for vulnerability in detected_vulnerabilities {
            println!(" - {}", vulnerability);
        }
    }
    Ok(())
}

// Main function to run the entire security analysis process
#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Initialize logging
    env_logger::init();
    
    // Parse command-line arguments
    let matches = ClapCommand::new("Security Analyzer")
        .version("1.0")
        .author("Your Name")
        .about("Analyzes files for vulnerabilities")
        .arg(Arg::new("tool_path")
            .long("tool")
            .takes_value(true)
            .required(true)
            .help("Path to the security analysis tool"))
        .arg(Arg::new("db_url")
            .long("db")
            .takes_value(true)
            .required(true)
            .help("URL of the vulnerability database"))
        .arg(Arg::new("files")
            .long("files")
            .takes_value(true)
            .multiple_values(true)
            .required(true)
            .help("Paths to files to analyze"))
        .get_matches();
    
    // Configuration settings
    let config = Config {
        tool_path: matches.value_of("tool_path").unwrap().to_string(),
        vulnerability_db_url: matches.value_of("db_url").unwrap().to_string(),
        file_paths: matches.values_of("files").unwrap().map(|s| s.to_string()).collect(),
    };
    
    // Analyze multiple files
    analyze_files(config.file_paths.clone(), &config).await?;
    
    // Fetch the latest vulnerability database from the remote URL
    let fetched_db = fetch_vulnerability_db(&config.vulnerability_db_url).await?;
    
    // Print the fetched vulnerability database for inspection
    info!("Fetched vulnerability database: {:?}", fetched_db);
    
    // Print a detailed summary of the analysis
    print_summary(&config.file_paths, &fetched_db)?;
    
    Ok(())
}