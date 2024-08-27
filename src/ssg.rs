use regex::Regex;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::collections::HashMap;
use std::io::{self, Write};
use std::ffi::OsStr;
use serde_json::json;
use std::fs::copy;

// Function to read the content of a file
fn read_file(path: &Path) -> io::Result<String> {
    fs::read_to_string(path)
}

// Function to write content to a file
fn write_file(path: &Path, content: &str) -> io::Result<()> {
    let mut file = fs::File::create(path)?;
    file.write_all(content.as_bytes())
}

// Function to replace placeholders in a template with actual content
fn apply_template(template: &str, content_map: &HashMap<String, String>) -> String {
    let mut result = template.to_string();
    for (key, value) in content_map {
        let re = Regex::new(&format!("{{{{{}}}}}", key)).unwrap();
        result = re.replace_all(&result, value).into_owned();
    }
    result
}

// Function to convert markdown text to HTML
fn markdown_to_html(markdown: &str) -> String {
    let mut html = markdown.to_string();

    let heading_re = Regex::new(r"(?m)^# (.+)$").unwrap();
    html = heading_re.replace_all(&html, "<h1>$1</h1>").into_owned();

    let heading2_re = Regex::new(r"(?m)^## (.+)$").unwrap();
    html = heading2_re.replace_all(&html, "<h2>$1</h2>").into_owned();

    let list_re = Regex::new(r"(?m)^\* (.+)$").unwrap();
    html = list_re.replace_all(&html, "<ul>\n<li>$1</li>\n</ul>").into_owned();

    let ordered_list_re = Regex::new(r"(?m)^\d+\. (.+)$").unwrap();
    html = ordered_list_re.replace_all(&html, "<ol>\n<li>$1</li>\n</ol>").into_owned();

    let code_re = Regex::new(r"```(.*?)```").unwrap();
    html = code_re.replace_all(&html, "<pre><code>$1</code></pre>").into_owned();

    let bold_re = Regex::new(r"\*\*(.*?)\*\*").unwrap();
    html = bold_re.replace_all(&html, "<strong>$1</strong>").into_owned();

    let italic_re = Regex::new(r"\*(.*?)\*").unwrap();
    html = italic_re.replace_all(&html, "<em>$1</em>").into_owned();

    let link_re = Regex::new(r"\[([^\]]+)\]\(([^\)]+)\)").unwrap();
    html = link_re.replace_all(&html, "<a href=\"$2\">$1</a>").into_owned();

    let image_re = Regex::new(r"!\[([^\]]*)\]\(([^\)]+)\)").unwrap();
    html = image_re.replace_all(&html, "<img src=\"$2\" alt=\"$1\" />").into_owned();

    html = format!("<html><body>{}</body></html>", html);
    html
}

// Function to extract metadata from markdown files
fn extract_metadata(markdown: &str) -> HashMap<String, String> {
    let mut metadata = HashMap::new();
    let re = Regex::new(r"(?m)^\s*([\w-]+):\s*(.*)$").unwrap();
    for cap in re.captures_iter(markdown) {
        if let (Some(key), Some(value)) = (cap.get(1), cap.get(2)) {
            metadata.insert(key.as_str().to_string(), value.as_str().to_string());
        }
    }
    metadata
}

// Function to copy static assets (e.g., images)
fn copy_assets(input_dir: &Path, output_dir: &Path) -> io::Result<()> {
    for entry in fs::read_dir(input_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            let new_output_dir = output_dir.join(path.file_name().unwrap());
            fs::create_dir_all(&new_output_dir)?;
            copy_assets(&path, &new_output_dir)?;
        } else if path.extension() == Some(OsStr::new("png")) ||
                  path.extension() == Some(OsStr::new("jpg")) ||
                  path.extension() == Some(OsStr::new("gif")) {
            let output_path = output_dir.join(path.file_name());
            copy(&path, &output_path)?;
        }
    }
    Ok(())
}

// Function to process markdown files and generate HTML
fn process_markdown_files(input_dir: &Path, output_dir: &Path) -> io::Result<()> {
    for entry in fs::read_dir(input_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            let new_output_dir = output_dir.join(path.file_name().unwrap());
            fs::create_dir_all(&new_output_dir)?;
            process_markdown_files(&path, &new_output_dir)?;
        } else if path.extension() == Some(OsStr::new("md")) {
            let content = read_file(&path)?;
            let metadata = extract_metadata(&content);
            let html_content = markdown_to_html(&content);
            let output_path = output_dir.join(path.file_stem().unwrap()).with_extension("html");
            write_file(&output_path, &html_content)?;

            let metadata_path = output_dir.join(path.file_stem().unwrap()).with_extension("json");
            let metadata_content = serde_json::to_string(&metadata)?;
            write_file(&metadata_path, &metadata_content)?;
        }
    }
    Ok(())
}

// Function to handle pagination
fn paginate_content(content: &str, items_per_page: usize) -> Vec<String> {
    let mut pages = Vec::new();
    let lines: Vec<&str> = content.lines().collect();
    let total_lines = lines.len();
    let mut start = 0;

    while start < total_lines {
        let end = (start + items_per_page).min(total_lines);
        let page_content = lines[start..end].join("\n");
        pages.push(page_content);
        start = end;
    }

    pages
}

// Function to generate the final site using a template
fn generate_site(template_path: &Path, output_dir: &Path, content_map: &HashMap<String, String>) -> io::Result<()> {
    let template_content = read_file(template_path)?;
    let final_html = apply_template(&template_content, content_map);
    write_file(&output_dir.join("index.html"), &final_html)?;
    Ok(())
}

// Main function to execute the SSG
fn main() -> io::Result<()> {
    env_logger::init();

    let input_dir = env::var("INPUT_DIR").unwrap_or_else(|_| "content".to_string());
    let output_dir = env::var("OUTPUT_DIR").unwrap_or_else(|_| "public".to_string());
    let template_path = env::var("TEMPLATE_PATH").unwrap_or_else(|_| "template.html".to_string());

    let input_dir_path = Path::new(&input_dir);
    let output_dir_path = Path::new(&output_dir);
    let template_path = Path::new(&template_path);

    if !output_dir_path.exists() {
        fs::create_dir_all(output_dir_path)?;
    }

    process_markdown_files(input_dir_path, output_dir_path)?;
    copy_assets(input_dir_path, output_dir_path)?;

    let mut content_map = HashMap::new();
    content_map.insert("title".to_string(), "My Static Site".to_string());
    content_map.insert("header".to_string(), "Welcome to My Static Site".to_string());
    content_map.insert("footer".to_string(), "© 2024 My Static Site".to_string());

    generate_site(template_path, output_dir_path, &content_map)?;

    println!("Static site generated successfully in {}", output_dir);
    Ok(())
}