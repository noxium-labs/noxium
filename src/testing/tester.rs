use serde::{Deserialize, Serialize};
use serde_json::json;
use reqwest::Client;
use std::fs;
use std::path::Path;
use html5ever::tendril::TendrilSink;
use html5ever::parse_document;
use html5ever::rcdom::{RcDom, Handle};
use cssparser::{Parser, Token};
use tokio;

#[derive(Debug, Serialize, Deserialize)]
struct TestCase {
    name: String,
    js: Option<String>,
    ts: Option<String>,
    html: Option<String>,
    css: Option<String>,
    assertions: Option<Vec<Assertion>>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Assertion {
    selector: String,
    property: String,
    expected_value: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct TestResult {
    test_name: String,
    passed: bool,
    details: String,
}

fn parse_html(html: &str) -> RcDom {
    let parser = parse_document(RcDom::default(), Default::default());
    parser.one(html)
}

fn extract_css_property(css: &str, property: &str) -> Option<String> {
    let mut parser = Parser::new(css);
    while let Ok(token) = parser.next() {
        match token {
            Token::Ident(id) if id == property => {
                if let Ok(Token::Colon) = parser.next() {
                    if let Ok(Token::Ident(value)) = parser.next() {
                        return Some(value);
                    }
                }
            },
            _ => {}
        }
    }
    None
}

async fn run_test(test_case: &TestCase) -> TestResult {
    let client = Client::new();
    let mut passed = true;
    let mut details = format!("Test '{}':\n", test_case.name);

    // Write HTML, JS, and CSS files
    if let Some(html) = &test_case.html {
        fs::write("test.html", html).expect("Unable to write HTML file");
    }
    if let Some(js) = &test_case.js {
        fs::write("test.js", js).expect("Unable to write JS file");
    }
    if let Some(css) = &test_case.css {
        fs::write("test.css", css).expect("Unable to write CSS file");
    }

    // Apply CSS and JavaScript (Placeholder logic)
    if let Some(js_code) = &test_case.js {
        details.push_str("\nJavaScript executed.");
    }
    if let Some(css_code) = &test_case.css {
        details.push_str("\nCSS applied.");
    }

    // Validate HTML through assertions
    if let Some(assertions) = &test_case.assertions {
        let response = client.get("http://localhost:8000/test.html").send().await.expect("Request failed");
        let body = response.text().await.expect("Failed to read response");
        let dom = parse_html(&body);

        for assertion in assertions {
            let css_property = if let Some(css_code) = &test_case.css {
                extract_css_property(css_code, &assertion.property)
            } else {
                None
            };

            if css_property.is_none() || css_property.unwrap() != assertion.expected_value {
                passed = false;
                details.push_str(&format!("\nAssertion failed for selector '{}'", assertion.selector));
            }
        }
    }

    TestResult {
        test_name: test_case.name.clone(),
        passed,
        details,
    }
}

fn load_test_cases(file_path: &str) -> Vec<TestCase> {
    let file_content = fs::read_to_string(file_path).expect("Unable to read file");
    serde_json::from_str(&file_content).expect("Unable to parse JSON")
}

#[tokio::main]
async fn main() {
    let test_cases = load_test_cases("data.json");

    for test_case in test_cases {
        let result = run_test(&test_case).await;
        println!("{:?}", result);
    }
}