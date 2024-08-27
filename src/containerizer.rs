use std::collections::HashMap;
use std::process::Command;
use std::io::{self, Write};
use std::fs;
use std::time::Duration;
use std::thread;

// Struct to represent a container
#[derive(Debug)]
struct Container {
    id: String,
    image: String,
    ports: HashMap<u16, u16>,
    environment: HashMap<String, String>,
}

impl Container {
    // Create a new container instance
    fn new(id: &str, image: &str) -> Self {
        Self {
            id: id.to_string(),
            image: image.to_string(),
            ports: HashMap::new(),
            environment: HashMap::new(),
        }
    }

    // Start the container
    fn start(&self) -> io::Result<()> {
        // Build port mappings argument for Docker
        let port_mappings: Vec<String> = self.ports.iter()
            .map(|(host_port, container_port)| format!("{}:{}", host_port, container_port))
            .collect();
        let port_mapping_arg = port_mappings.join(" ");

        // Build environment variables argument for Docker
        let env_vars: Vec<String> = self.environment.iter()
            .map(|(key, value)| format!("{}={}", key, value))
            .collect();
        let env_vars_arg = env_vars.join(" ");

        // Run Docker container
        let output = Command::new("docker")
            .arg("run")
            .arg("-d") // Run container in detached mode
            .arg("--name").arg(&self.id)
            .arg("-p").arg(port_mapping_arg)
            .args(env_vars.iter().map(|var| ["-e", var]).flatten())
            .arg(&self.image)
            .output()?;

        // Check if Docker command was successful
        if !output.status.success() {
            return Err(io::Error::new(io::ErrorKind::Other, "Failed to start container"));
        }
        Ok(())
    }

    // Stop the container
    fn stop(&self) -> io::Result<()> {
        let output = Command::new("docker")
            .arg("stop")
            .arg(&self.id)
            .output()?;
        if !output.status.success() {
            return Err(io::Error::new(io::ErrorKind::Other, "Failed to stop container"));
        }
        Ok(())
    }

    // Remove the container
    fn remove(&self) -> io::Result<()> {
        let output = Command::new("docker")
            .arg("rm")
            .arg(&self.id)
            .output()?;
        if !output.status.success() {
            return Err(io::Error::new(io::ErrorKind::Other, "Failed to remove container"));
        }
        Ok(())
    }

    // Set port mappings for the container
    fn set_ports(&mut self, ports: HashMap<u16, u16>) {
        self.ports = ports;
    }

    // Set environment variables for the container
    fn set_environment(&mut self, environment: HashMap<String, String>) {
        self.environment = environment;
    }

    // Get the logs of the container
    fn logs(&self) -> io::Result<String> {
        let output = Command::new("docker")
            .arg("logs")
            .arg(&self.id)
            .output()?;
        if !output.status.success() {
            return Err(io::Error::new(io::ErrorKind::Other, "Failed to get logs"));
        }
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }

    // Check if the container is running
    fn is_running(&self) -> io::Result<bool> {
        let output = Command::new("docker")
            .arg("ps")
            .arg("-q")
            .arg("-f").arg(format!("name={}", self.id))
            .output()?;
        Ok(!output.stdout.is_empty())
    }
}

fn main() -> io::Result<()> {
    // Create a container with ID and image
    let mut container = Container::new("my_website_container", "nginx:latest");

    // Set port mappings (host_port -> container_port)
    let mut ports = HashMap::new();
    ports.insert(8080, 80);
    container.set_ports(ports);

    // Set environment variables (e.g., setting a timezone)
    let mut env_vars = HashMap::new();
    env_vars.insert("TZ".to_string(), "UTC".to_string());
    container.set_environment(env_vars);

    // Start the container
    container.start()?;
    println!("Container started");

    // Wait and check container status
    thread::sleep(Duration::from_secs(5));
    if container.is_running()? {
        println!("Container is running");
    } else {
        println!("Container is not running");
    }

    // Print container logs
    let logs = container.logs()?;
    println!("Container logs:\n{}", logs);

    // Simulate doing work
    println!("Press Enter to stop the container...");
    let _ = io::stdin().read_line(&mut String::new())?;

    // Stop and remove the container
    container.stop()?;
    println!("Container stopped");
    container.remove()?;
    println!("Container removed");

    Ok(())
}