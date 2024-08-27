use reqwest::blocking::Client;
use scraper::{Html, Selector};
use std::collections::HashMap;

fn main() {
    let url = "https://example.com"; // Replace with the URL you want to analyze

    // Analyze the SEO and print the results or errors
    match analyze_seo(url) {
        Ok(result) => println!("{:#?}", result), // Pretty-print the SEO results
        Err(e) => println!("Error: {}", e), // Print any errors encountered
    }
}

// Function to analyze various SEO aspects of a webpage
fn analyze_seo(url: &str) -> Result<SeoResult, Box<dyn std::error::Error>> {
    let client = Client::new(); // Create a new HTTP client
    let response = client.get(url).send()?.text()?; // Send a GET request and get the response text

    let document = Html::parse_document(&response); // Parse the HTML content into a document structure

    // Extract various SEO elements using helper functions
    let title = get_title(&document);
    let meta_description = get_meta_description(&document);
    let heading_counts = get_heading_counts(&document);
    let image_alt_count = get_image_alt_count(&document);
    let word_count = get_word_count(&document);
    let internal_links = get_internal_links(&document, url);
    let external_links = get_external_links(&document, url);
    let meta_keywords = get_meta_keywords(&document);
    let content_length = get_content_length(&document);
    let has_robots_txt = check_robots_txt(url)?;
    let has_sitemap = check_sitemap(url)?;
    let meta_tag_count = count_meta_tags(&document);
    let external_js_css_count = count_external_js_css(&document);
    let nofollow_links_count = count_nofollow_links(&document);

    // Return all collected SEO data encapsulated in a structured format
    Ok(SeoResult {
        title,
        meta_description,
        heading_counts,
        image_alt_count,
        word_count,
        internal_links,
        external_links,
        meta_keywords,
        content_length,
        has_robots_txt,
        has_sitemap,
        meta_tag_count,
        external_js_css_count,
        nofollow_links_count,
    })
}

// Function to extract the title of the webpage
fn get_title(document: &Html) -> Option<String> {
    let selector = Selector::parse("title").unwrap(); // Create a selector for the <title> tag
    document.select(&selector).next().map(|e| e.inner_html()) // Extract the inner HTML of the <title> tag
}

// Function to extract the meta description of the webpage
fn get_meta_description(document: &Html) -> Option<String> {
    let selector = Selector::parse(r#"meta[name="description"]"#).unwrap(); // Create a selector for <meta name="description">
    document
        .select(&selector)
        .next()
        .and_then(|e| e.value().attr("content").map(String::from)) // Extract the content attribute of the meta tag
}

// Function to count the number of heading tags (h1 to h6) on the webpage
fn get_heading_counts(document: &Html) -> Vec<(String, usize)> {
    let mut counts = vec![]; // Vector to store counts of each heading type
    for level in 1..=6 { // Loop through heading levels from h1 to h6
        let selector = Selector::parse(&format!("h{}", level)).unwrap(); // Create a selector for each heading level
        let count = document.select(&selector).count(); // Count the number of each heading level
        counts.push((format!("h{}", level), count)); // Store the count in the vector
    }
    counts // Return the vector containing heading counts
}

// Function to count the number of images with alt attributes on the webpage
fn get_image_alt_count(document: &Html) -> usize {
    let selector = Selector::parse("img").unwrap(); // Create a selector for the <img> tag
    document
        .select(&selector)
        .filter(|img| img.value().attr("alt").is_some()) // Filter images that have an "alt" attribute
        .count() // Count the number of images with an alt attribute
}

// Function to count the number of words on the webpage
fn get_word_count(document: &Html) -> usize {
    let selector = Selector::parse("body").unwrap(); // Create a selector for the <body> tag
    let body = document.select(&selector).next(); // Select the body element
    if let Some(body) = body {
        let text = body.text().collect::<Vec<_>>().join(" "); // Collect all text nodes into a single string
        text.split_whitespace().count() // Split the text by whitespace and count the words
    } else {
        0 // Return 0 if the body is not found
    }
}

// Function to count the number of internal links on the webpage
fn get_internal_links(document: &Html, base_url: &str) -> usize {
    let selector = Selector::parse("a[href]").unwrap(); // Create a selector for anchor tags with href attributes
    document
        .select(&selector)
        .filter(|a| {
            if let Some(href) = a.value().attr("href") {
                href.starts_with(base_url) // Check if the href starts with the base URL
            } else {
                false
            }
        })
        .count() // Count the number of internal links
}

// Function to count the number of external links on the webpage
fn get_external_links(document: &Html, base_url: &str) -> usize {
    let selector = Selector::parse("a[href]").unwrap(); // Create a selector for anchor tags with href attributes
    document
        .select(&selector)
        .filter(|a| {
            if let Some(href) = a.value().attr("href") {
                href.starts_with("http") && !href.starts_with(base_url) // Check if the href starts with "http" and is not internal
            } else {
                false
            }
        })
        .count() // Count the number of external links
}

// Function to extract meta keywords from the webpage
fn get_meta_keywords(document: &Html) -> Option<String> {
    let selector = Selector::parse(r#"meta[name="keywords"]"#).unwrap(); // Create a selector for <meta name="keywords">
    document
        .select(&selector)
        .next()
        .and_then(|e| e.value().attr("content").map(String::from)) // Extract the content attribute of the meta tag
}

// Function to calculate the length of content on the webpage
fn get_content_length(document: &Html) -> usize {
    let selector = Selector::parse("body").unwrap(); // Create a selector for the <body> tag
    let body = document.select(&selector).next(); // Select the body element
    if let Some(body) = body {
        let text = body.text().collect::<Vec<_>>().join(" "); // Collect all text nodes into a single string
        text.len() // Return the length of the text
    } else {
        0 // Return 0 if the body is not found
    }
}

// Function to check if a site has a robots.txt file
fn check_robots_txt(url: &str) -> Result<bool, Box<dyn std::error::Error>> {
    let robots_txt_url = format!("{}/robots.txt", url); // Construct the URL for robots.txt
    let client = Client::new();
    let response = client.get(&robots_txt_url).send()?; // Send a GET request to check if robots.txt exists
    Ok(response.status().is_success()) // Return true if the request is successful
}

// Function to check if a site has a sitemap
fn check_sitemap(url: &str) -> Result<bool, Box<dyn std::error::Error>> {
    let sitemap_url = format!("{}/sitemap.xml", url); // Construct the URL for sitemap.xml
    let client = Client::new();
    let response = client.get(&sitemap_url).send()?; // Send a GET request to check if sitemap.xml exists
    Ok(response.status().is_success()) // Return true if the request is successful
}

// Function to count the number of meta tags on the webpage
fn count_meta_tags(document: &Html) -> usize {
    let selector = Selector::parse("meta").unwrap(); // Create a selector for the <meta> tag
    document.select(&selector).count() // Count the number of meta tags
}

// Function to count the number of external JavaScript and CSS files on the webpage
fn count_external_js_css(document: &Html) -> HashMap<String, usize> {
    let mut count = HashMap::new(); // Create a hashmap to store counts of JavaScript and CSS files
    count.insert("js".to_string(), 0); // Initialize JavaScript file count to 0
    count.insert("css".to_string(), 0); // Initialize CSS file count to 0

    let script_selector = Selector::parse("script[src]").unwrap(); // Create a selector for external JavaScript files
    let link_selector = Selector::parse(r#"link[rel="stylesheet"]"#).unwrap(); // Create a selector for external CSS files

    for _ in document.select(&script_selector) {
        *count.get_mut("js").unwrap() += 1; // Increment the JavaScript file count
    }

    for _ in document.select(&link_selector) {
        *count.get_mut("css").unwrap() += 1; // Increment the CSS file count
    }

    count // Return the hashmap containing counts of JavaScript and CSS files
}

// Function to count the number of links with "nofollow" attribute on the webpage
fn count_nofollow_links(document: &Html) -> usize {
    let selector = Selector::parse(r#"a[rel="nofollow"]"#).unwrap(); // Create a selector for anchor tags with rel="nofollow"
    document.select(&selector).count() // Count the number of nofollow links
}

// Struct to encapsulate the SEO results
#[derive(Debug)]
struct SeoResult {
    title: Option<String>, // Title of the webpage
    meta_description: Option<String>, // Meta description of the webpage
    heading_counts: Vec<(String, usize)>, // Counts of heading tags (h1 to h6)
    image_alt_count: usize, // Count of images with alt attributes
    word_count: usize, // Count of words on the webpage
    internal_links: usize, // Count of internal links on the webpage
    external_links: usize, // Count of external links on the webpage
    meta_keywords: Option<String>, // Meta keywords of the webpage
    content_length: usize, // Length of the content on the webpage
    has_robots_txt: bool, // Indicates if the site has a robots.txt file
    has_sitemap: bool, // Indicates if the site has a sitemap.xml file
    meta_tag_count: usize, // Count of meta tags on the webpage
    external_js_css_count: HashMap<String, usize>, // Counts of external JavaScript and CSS files
    nofollow_links_count: usize, // Count of links with "nofollow" attribute
}