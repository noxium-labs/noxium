use reqwest::blocking::get;
use select::document::Document;
use select::predicate::{Name, Predicate};
use std::error::Error;
use url::Url;
use regex::Regex;
use serde_json::Value;
use std::collections::HashSet;
use std::time::Instant;

/// Fetch the HTML content from a URL
fn fetch_html(url: &str) -> Result<String, Box<dyn Error>> {
    let response = get(url)?;
    if !response.status().is_success() {
        return Err(format!("Failed to fetch {}: {}", url, response.status()).into());
    }
    Ok(response.text()?)
}

/// Extract and print the title tag content
fn print_title(document: &Document) {
    if let Some(title) = document.find(Name("title")).next() {
        println!("Title: {}", title.text());
    } else {
        println!("Title tag not found");
    }
}

/// Extract and print the meta description
fn print_meta_description(document: &Document) {
    if let Some(description) = document.find(Name("meta")).filter(|n| n.attr("name") == Some("description")).next() {
        if let Some(content) = description.attr("content") {
            println!("Meta Description: {}", content);
        }
    } else {
        println!("Meta Description tag not found");
    }
}

/// Extract and print header tags (h1, h2, h3, h4, h5, h6)
fn print_headers(document: &Document) {
    for header in ["h1", "h2", "h3", "h4", "h5", "h6"].iter() {
        for node in document.find(Name(header)) {
            println!("{}: {}", header.to_uppercase(), node.text());
        }
    }
}

/// Extract and print canonical URL
fn print_canonical_url(document: &Document) {
    if let Some(canonical) = document.find(Name("link")).filter(|n| n.attr("rel") == Some("canonical")).next() {
        if let Some(href) = canonical.attr("href") {
            println!("Canonical URL: {}", href);
        }
    } else {
        println!("Canonical URL tag not found");
    }
}

/// Extract and print alt attributes of images
fn print_image_alts(document: &Document) {
    for img in document.find(Name("img")) {
        if let Some(alt) = img.attr("alt") {
            println!("Image Alt: {}", alt);
        } else {
            println!("Image with no alt attribute found");
        }
    }
}

/// Check for broken links by making HTTP requests and printing status codes
fn check_broken_links(document: &Document, base_url: &str) -> Result<(), Box<dyn Error>> {
    for link in document.find(Name("a")) {
        if let Some(href) = link.attr("href") {
            let absolute_url = resolve_url(base_url, href)?;
            let response = get(&absolute_url)?;
            if !response.status().is_success() {
                println!("Broken link: {} (Status: {})", absolute_url, response.status());
            }
        }
    }
    Ok(())
}

/// Resolve a relative URL to an absolute URL using the base URL
fn resolve_url(base_url: &str, relative_url: &str) -> Result<String, Box<dyn Error>> {
    let base = Url::parse(base_url)?;
    let resolved_url = base.join(relative_url)?;
    Ok(resolved_url.to_string())
}

/// Print the response time of the URL
fn print_response_time(url: &str) -> Result<(), Box<dyn Error>> {
    let start_time = Instant::now();
    let response = get(url)?;
    let duration = start_time.elapsed();
    if response.status().is_success() {
        println!("Response time for {}: {:?}", url, duration);
    } else {
        println!("Failed to fetch {}: {}", url, response.status());
    }
    Ok(())
}

/// Print all meta tags for further analysis
fn print_meta_tags(document: &Document) {
    for meta in document.find(Name("meta")) {
        if let Some(name) = meta.attr("name") {
            if let Some(content) = meta.attr("content") {
                println!("Meta Name: {} Content: {}", name, content);
            }
        } else if let Some(property) = meta.attr("property") {
            if let Some(content) = meta.attr("content") {
                println!("Meta Property: {} Content: {}", property, content);
            }
        }
    }
}

/// Check if a page has a `robots` meta tag
fn check_robots_tag(document: &Document) {
    if let Some(robots) = document.find(Name("meta")).filter(|n| n.attr("name") == Some("robots")).next() {
        if let Some(content) = robots.attr("content") {
            println!("Robots Meta Tag: {}", content);
        }
    } else {
        println!("Robots meta tag not found");
    }
}

/// Check for the presence of Open Graph tags
fn check_open_graph_tags(document: &Document) {
    let og_tags = ["og:title", "og:description", "og:image", "og:url"];
    for tag in og_tags.iter() {
        if let Some(og_tag) = document.find(Name("meta")).filter(|n| n.attr("property") == Some(*tag)).next() {
            if let Some(content) = og_tag.attr("content") {
                println!("Open Graph {}: {}", tag, content);
            }
        } else {
            println!("Open Graph {} tag not found", tag);
        }
    }
}

/// Simulate backlink analysis (dummy implementation)
fn analyze_backlinks(url: &str) -> Result<(), Box<dyn Error>> {
    println!("Analyzing backlinks for {}", url);
    // Dummy implementation: in reality, this would use an external service or API.
    println!("Backlink analysis not implemented.");
    Ok(())
}

/// Simulate content matching based on search queries (dummy implementation)
fn match_content(search_query: &str, content: &str) {
    println!("Matching content for search query: {}", search_query);
    let content_lower = content.to_lowercase();
    if content_lower.contains(&search_query.to_lowercase()) {
        println!("Content matches the search query.");
    } else {
        println!("Content does not match the search query.");
    }
}

/// Extract page content and analyze it
fn analyze_page_content(document: &Document) -> String {
    document.find(Name("body")).next().map_or_else(
        || "No body content found".to_string(),
        |body| body.text(),
    )
}

fn main() -> Result<(), Box<dyn Error>> {
    // Replace with the URL you want to analyze
    let url = "https://example.com";
    
    // Fetch the HTML content
    let html_content = fetch_html(url)?;
    let document = Document::from(html_content.clone());
    
    // Print various SEO elements
    print_title(&document);
    print_meta_description(&document);
    print_headers(&document);
    print_canonical_url(&document);
    print_image_alts(&document);
    
    // Check for broken links
    check_broken_links(&document, url)?;
    
    // Print the response time
    print_response_time(url)?;
    
    // Print all meta tags
    print_meta_tags(&document);
    
    // Check for robots meta tag
    check_robots_tag(&document);
    
    // Check for Open Graph tags
    check_open_graph_tags(&document);
    
    // Analyze backlinks
    analyze_backlinks(url)?;
    
    // Analyze page content
    let page_content = analyze_page_content(&document);
    
    // Simulate content matching based on a search query
    let search_query = "example";
    match_content(search_query, &page_content);
    
    Ok(())
}