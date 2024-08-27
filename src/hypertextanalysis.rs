// Import necessary crates for HTML parsing, file handling, HTTP requests, and asynchronous execution
use scraper::{Html, Selector}; // For HTML parsing and element selection
use std::collections::HashMap; // Standard library HashMap for storing tag and attribute counts
use std::fmt; // For custom formatting of output
use std::fs; // For reading HTML content from files
use std::io; // For handling input/output errors
use reqwest; // For making HTTP requests to fetch HTML content
use std::env; // For handling environment variables

// Define a struct to hold the results of the HTML analysis
// This struct will be responsible for counting and displaying tag frequencies, attributes, nesting levels, and text content
struct AnalysisResult {
    tag_count: HashMap<String, usize>, // HashMap to store the count of each HTML tag
    attribute_count: HashMap<String, usize>, // HashMap to store the count of each HTML attribute
    tag_nesting_level: HashMap<String, usize>, // HashMap to store the maximum nesting level of each tag
    total_text_content: String, // String to store the accumulated text content from the HTML
    unique_tags: HashMap<String, usize>, // HashMap to store unique tags and their occurrences
    attribute_per_tag: HashMap<String, HashMap<String, usize>>, // Nested HashMap to store attribute counts per tag
}

// Implement methods for the AnalysisResult struct
impl AnalysisResult {
    // Constructor method to create a new instance of AnalysisResult
    fn new() -> Self {
        Self {
            tag_count: HashMap::new(), // Initialize tag_count as an empty HashMap
            attribute_count: HashMap::new(), // Initialize attribute_count as an empty HashMap
            tag_nesting_level: HashMap::new(), // Initialize tag_nesting_level as an empty HashMap
            total_text_content: String::new(), // Initialize total_text_content as an empty string
            unique_tags: HashMap::new(), // Initialize unique_tags as an empty HashMap
            attribute_per_tag: HashMap::new(), // Initialize attribute_per_tag as an empty nested HashMap
        }
    }

    // Method to analyze the provided HTML string and update tag and attribute counts
    fn analyze(&mut self, html: &str) {
        let document = Html::parse_document(html); // Parse the HTML content into a document object
        let selector = Selector::parse("*").unwrap(); // Create a Selector to select all elements

        let mut tag_stack: Vec<String> = Vec::new(); // Track the current nesting level of tags

        for element in document.select(&selector) {
            let tag_name = element.value().name().to_string(); // Get the tag name

            // Update tag count
            let count = self.tag_count.entry(tag_name.clone()).or_insert(0);
            *count += 1;

            // Update unique tags
            let unique_count = self.unique_tags.entry(tag_name.clone()).or_insert(0);
            *unique_count += 1;

            // Update tag nesting level
            let nesting_level = tag_stack.len();
            let max_level = self.tag_nesting_level.entry(tag_name.clone()).or_insert(nesting_level);
            *max_level = std::cmp::max(*max_level, nesting_level);

            // Update tag stack
            tag_stack.push(tag_name.clone());

            // Iterate over all attributes of the current element
            for attr in element.attributes() {
                let attr_name = attr.key().to_string();

                // Update attribute count
                let attr_count = self.attribute_count.entry(attr_name.clone()).or_insert(0);
                *attr_count += 1;

                // Update attribute count per tag
                let tag_attr_map = self.attribute_per_tag
                    .entry(tag_name.clone())
                    .or_insert_with(HashMap::new);
                let tag_attr_count = tag_attr_map.entry(attr_name.clone()).or_insert(0);
                *tag_attr_count += 1;
            }

            // Extract and accumulate the text content of the element
            let text_content = element.text().collect::<Vec<_>>().concat();
            self.total_text_content.push_str(&text_content);

            // Remove the current tag from the stack after processing its children
            tag_stack.pop();
        }
    }

    // Method to print the results of the HTML analysis
    fn print_results(&self) {
        println!("Tag Counts:");
        for (tag, count) in &self.tag_count {
            println!("Tag: {}, Count: {}", tag, count);
        }

        println!("\nUnique Tags:");
        for (tag, count) in &self.unique_tags {
            println!("Tag: {}, Unique Occurrences: {}", tag, count);
        }

        println!("\nAttribute Counts:");
        for (attr, count) in &self.attribute_count {
            println!("Attribute: {}, Count: {}", attr, count);
        }

        println!("\nAttribute Counts Per Tag:");
        for (tag, attrs) in &self.attribute_per_tag {
            println!("Tag: {}", tag);
            for (attr, count) in attrs {
                println!("  Attribute: {}, Count: {}", attr, count);
            }
        }

        println!("\nTag Nesting Levels:");
        for (tag, level) in &self.tag_nesting_level {
            println!("Tag: {}, Max Nesting Level: {}", tag, level);
        }

        println!("\nTotal Text Content:");
        println!("{}", self.total_text_content);
    }
}

// Implement the Display trait for AnalysisResult to allow custom formatted output
impl fmt::Display for AnalysisResult {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "Tag Counts:")?;
        for (tag, count) in &self.tag_count {
            writeln!(f, "Tag: {}, Count: {}", tag, count)?;
        }

        writeln!(f, "\nUnique Tags:")?;
        for (tag, count) in &self.unique_tags {
            writeln!(f, "Tag: {}, Unique Occurrences: {}", tag, count)?;
        }

        writeln!(f, "\nAttribute Counts:")?;
        for (attr, count) in &self.attribute_count {
            writeln!(f, "Attribute: {}, Count: {}", attr, count)?;
        }

        writeln!(f, "\nAttribute Counts Per Tag:")?;
        for (tag, attrs) in &self.attribute_per_tag {
            writeln!(f, "Tag: {}", tag)?;
            for (attr, count) in attrs {
                writeln!(f, "  Attribute: {}, Count: {}", attr, count)?;
            }
        }

        writeln!(f, "\nTag Nesting Levels:")?;
        for (tag, level) in &self.tag_nesting_level {
            writeln!(f, "Tag: {}, Max Nesting Level: {}", tag, level)?;
        }

        writeln!(f, "\nTotal Text Content:")?;
        writeln!(f, "{}", self.total_text_content)?;

        Ok(())
    }
}

// Function to fetch HTML content from a URL
// Takes a URL as a string and returns the HTML content as a String
async fn fetch_html_from_url(url: &str) -> Result<String, reqwest::Error> {
    let response = reqwest::get(url).await?; // Send HTTP GET request
    let html = response.text().await?; // Extract HTML text from the response
    Ok(html)
}

// Function to read HTML content from a file
// Takes a file path as a string and returns the HTML content as a String
fn read_html_from_file(file_path: &str) -> Result<String, io::Error> {
    fs::read_to_string(file_path) // Read the file content into a string
}

// Function to process HTML content from different sources
// Takes a source type (file or URL) and a source string (file path or URL)
// Returns the HTML content as a String or an error
async fn process_html_source(source_type: &str, source: &str) -> Result<String, Box<dyn std::error::Error>> {
    match source_type {
        "file" => {
            let html = read_html_from_file(source)?;
            Ok(html)
        }
        "url" => {
            let html = fetch_html_from_url(source).await?;
            Ok(html)
        }
        _ => Err("Invalid source type".into()),
    }
}

// Main function to demonstrate the functionality of the analysis tool
#[tokio::main]
async fn main() {
    // Example of analyzing HTML content from a string
    let html_string = "<html><head><title>Test</title></head><body><h1>Hello</h1><p id=\"para1\">World</p></body></html>";
    
    let mut analysis_result = AnalysisResult::new();
    analysis_result.analyze(html_string);
    println!("{}", analysis_result);

    // Read HTML content from a file
    let file_path = "path/to/your/file.html";
    match read_html_from_file(file_path) {
        Ok(html) => {
            let mut file_analysis_result = AnalysisResult::new();
            file_analysis_result.analyze(&html);
            println!("{}", file_analysis_result);
        }
        Err(e) => eprintln!("Error reading file: {}", e),
    }

    // Fetch HTML content from a URL
    let url = "https://example.com";
    match fetch_html_from_url(url).await {
        Ok(html) => {
            let mut url_analysis_result = AnalysisResult::new();
            url_analysis_result.analyze(&html);
            println!("{}", url_analysis_result);
        }
        Err(e) => eprintln!("Error fetching URL: {}", e),
    }

    // Example of processing HTML content from different sources
    let source_type = env::var("SOURCE_TYPE").unwrap_or_else(|_| "file".to_string());
    let source = env::var("SOURCE").unwrap_or_else(|_| "path/to/your/file.html".to_string());

    match process_html_source(&source_type, &source).await {
        Ok(html) => {
            let mut source_analysis_result = AnalysisResult::new();
            source_analysis_result.analyze(&html);
            println!("{}", source_analysis_result);
        }
        Err(e) => eprintln!("Error processing source: {}", e),
    }
}