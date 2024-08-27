use notify::{watcher, RecursiveMode, Watcher};
use regex::Regex;
use std::env;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::mpsc::channel;
use std::time::Duration;
use toml::de::from_str as toml_from_str;

const CONFIG_FILE: &str = "build.toml";

#[derive(Debug, serde::Deserialize)]
struct BuildConfig {
    typescript: Option<ConfigOptions>,
    javascript: Option<ConfigOptions>,
    css: Option<ConfigOptions>,
    html: Option<ConfigOptions>,
    images: Option<ConfigOptions>,
    custom_commands: Option<Vec<String>>,
}

#[derive(Debug, serde::Deserialize)]
struct ConfigOptions {
    input: String,
    output: String,
    options: Option<Vec<String>>,
}

fn main() {
    // Load configuration
    let config = match load_config(CONFIG_FILE) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Failed to load configuration: {:?}", e);
            return;
        }
    };

    // Create a channel for file system events
    let (tx, rx) = channel();

    // Create a watcher to monitor changes in the "src" directory
    let mut watcher = watcher(tx, Duration::from_secs(2)).unwrap();
    watcher.watch("src", RecursiveMode::Recursive).unwrap();
    println!("Watching for changes in the 'src' directory...");

    // Main loop to handle file system events
    loop {
        match rx.recv() {
            Ok(_) => {
                // Rebuild when changes are detected
                println!("Changes detected. Rebuilding...");
                build(&config);
            }
            Err(e) => eprintln!("Watch error: {:?}", e),
        }
    }
}

fn load_config(file: &str) -> Result<BuildConfig, Box<dyn std::error::Error>> {
    let content = fs::read_to_string(file)?;
    let config: BuildConfig = toml_from_str(&content)?;
    Ok(config)
}

fn build(config: &BuildConfig) {
    // Compile TypeScript to JavaScript if configured
    if let Some(ts) = &config.typescript {
        if let Err(e) = Command::new("tsc")
            .arg("--outDir")
            .arg(&ts.output)
            .status()
        {
            eprintln!("Failed to compile TypeScript: {:?}", e);
        } else {
            println!("TypeScript compilation complete.");
        }
    }

    // Minify JavaScript files if configured
    if let Some(js) = &config.javascript {
        if let Err(e) = Command::new("terser")
            .arg(&js.input)
            .arg("--compress")
            .arg("--mangle")
            .arg("--output")
            .arg(&js.output)
            .status()
        {
            eprintln!("Failed to minify JavaScript: {:?}", e);
        } else {
            println!("JavaScript minification complete.");
        }
    }

    // Minify CSS files if configured
    if let Some(css) = &config.css {
        if let Err(e) = Command::new("cleancss")
            .arg(&css.input)
            .arg("-o")
            .arg(&css.output)
            .status()
        {
            eprintln!("Failed to minify CSS: {:?}", e);
        } else {
            println!("CSS minification complete.");
        }
    }

    // Copy HTML files if configured
    if let Some(html) = &config.html {
        copy_files(&html.input, &html.output, "HTML");
    }

    // Copy image files if configured
    if let Some(images) = &config.images {
        copy_files(&images.input, &images.output, "Images");
    }

    // Run custom commands if configured
    if let Some(commands) = &config.custom_commands {
        for cmd in commands {
            if let Err(e) = Command::new("sh").arg("-c").arg(cmd).status() {
                eprintln!("Failed to run custom command '{}': {:?}", cmd, e);
            } else {
                println!("Custom command '{}' executed successfully.", cmd);
            }
        }
    }

    println!("Build complete.");
}

fn copy_files(input_pattern: &str, output_dir: &str, file_type: &str) {
    let re = Regex::new(&input_pattern.replace("**/*", ".*")).unwrap();
    let paths = fs::read_dir("src").unwrap();

    for entry in paths {
        let entry = entry.unwrap();
        let path = entry.path();
        let filename = path.file_name().unwrap().to_str().unwrap();

        if re.is_match(filename) {
            let output_path = Path::new(output_dir).join(filename);
            if let Err(e) = fs::copy(&path, &output_path) {
                eprintln!("Failed to copy {} file '{}': {:?}", file_type, filename, e);
            } else {
                println!("Copied {} file '{}'.", file_type, filename);
            }
        }
    }
}