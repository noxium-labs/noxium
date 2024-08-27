use reqwest::blocking::get;
use reqwest::StatusCode;
use scraper::{Html, Selector};
use log::{info, error};
use std::collections::HashMap;

// Initialize logger
fn init_logger() {
    env_logger::init();
}

// Main function to fetch webpage and extract detailed information
fn main() {
    init_logger();

    // URL to fetch
    let url = "https://www.example.com";

    // Fetch the webpage content
    match fetch_webpage(url) {
        Ok(body) => {
            // Parse and extract information from the HTML body
            let details = extract_webpage_details(&body);
            display_details(&details);
        },
        Err(e) => {
            error!("Error fetching webpage: {}", e);
        }
    }
}

// Function to fetch the webpage content
fn fetch_webpage(url: &str) -> Result<String, reqwest::Error> {
    info!("Fetching webpage: {}", url);

    // Send a blocking GET request
    let response = get(url)?;

    // Check if the response status is success
    match response.status() {
        StatusCode::OK => {
            info!("Successfully fetched webpage.");
            response.text()
        },
        status => {
            error!("Failed to fetch webpage. Status: {}", status);
            Err(reqwest::Error::new(
                reqwest::ErrorKind::Status,
                format!("Failed to fetch webpage: {}", status),
            ))
        }
    }
}

// Function to extract details from the HTML body
fn extract_webpage_details(body: &str) -> HashMap<String, Vec<String>> {
    let mut details: HashMap<String, Vec<String>> = HashMap::new();
    let document = Html::parse_document(body);

    // Extract the title
    let title_selector = Selector::parse("title").unwrap();
    let title = document.select(&title_selector).next().map_or("No title found".to_string(), |e| e.inner_html());
    details.entry("Title".to_string()).or_default().push(title);

    // Extract meta tags
    extract_meta_tags(&document, &mut details);

    // Extract all links
    extract_links(&document, &mut details);

    // Extract all images
    extract_images(&document, &mut details);

    details
}

// Function to extract meta tags from the document
fn extract_meta_tags(document: &Html, details: &mut HashMap<String, Vec<String>>) {
    let meta_selector = Selector::parse("meta").unwrap();
    for meta in document.select(&meta_selector) {
        if let Some(name) = meta.value().attr("name") {
            if let Some(content) = meta.value().attr("content") {
                details.entry(format!("Meta - {}", name)).or_default().push(content.to_string());
            }
        }
        if let Some(property) = meta.value().attr("property") {
            if let Some(content) = meta.value().attr("content") {
                details.entry(format!("Meta - {}", property)).or_default().push(content.to_string());
            }
        }
    }
}

// Function to extract all hyperlinks from the document
fn extract_links(document: &Html, details: &mut HashMap<String, Vec<String>>) {
    let link_selector = Selector::parse("a").unwrap();
    for link in document.select(&link_selector) {
        if let Some(href) = link.value().attr("href") {
            details.entry("Links".to_string()).or_default().push(href.to_string());
        }
    }
}

// Function to extract all images from the document
fn extract_images(document: &Html, details: &mut HashMap<String, Vec<String>>) {
    let img_selector = Selector::parse("img").unwrap();
    for img in document.select(&img_selector) {
        if let Some(src) = img.value().attr("src") {
            details.entry("Images".to_string()).or_default().push(src.to_string());
        }
    }
}

// Function to display extracted details
fn display_details(details: &HashMap<String, Vec<String>>) {
    for (key, values) in details {
        println!("{}:", key);
        for value in values {
            println!("  - {}", value);
        }
    }
}