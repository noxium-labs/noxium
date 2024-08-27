use kuchiki::traits::*;
use kuchiki::parse_html;
use structopt::StructOpt;
use std::fs;

#[derive(Debug, StructOpt)]
#[structopt(name = "element_blocker", about = "A complex HTML element blocker in Rust.")]
struct Opt {
    /// Input HTML file
    #[structopt(short, long)]
    input: String,

    /// Output HTML file
    #[structopt(short, long)]
    output: String,

    /// Block elements by tag name
    #[structopt(long)]
    tag: Option<Vec<String>>,

    /// Block elements by class name
    #[structopt(long)]
    class: Option<Vec<String>>,

    /// Block elements by ID
    #[structopt(long)]
    id: Option<Vec<String>>,

    /// Block elements by attribute (format: key=value)
    #[structopt(long)]
    attr: Option<Vec<String>>,
}

fn main() {
    let opt = Opt::from_args();
    
    // Read the input HTML file
    let html = fs::read_to_string(&opt.input).expect("Unable to read input file");

    // Parse the HTML
    let document = parse_html().one(html);

    // Get the elements to block
    let tags = opt.tag.unwrap_or_default();
    let classes = opt.class.unwrap_or_default();
    let ids = opt.id.unwrap_or_default();
    let attrs = opt.attr.unwrap_or_default();

    // Function to match elements based on conditions
    let should_block = |node: &kuchiki::NodeData| -> bool {
        if let Some(tag_name) = node.as_element().map(|e| e.name.local.as_ref().to_string()) {
            if tags.contains(&tag_name) {
                return true;
            }
        }

        if let Some(attrs) = node.as_element().map(|e| e.attributes.borrow()) {
            for class in &classes {
                if attrs.get("class").map_or(false, |v| v.split_whitespace().any(|c| c == class)) {
                    return true;
                }
            }

            for id in &ids {
                if attrs.get("id").map_or(false, |v| v == id) {
                    return true;
                }
            }

            for attr in &attrs {
                let mut parts = attr.splitn(2, '=');
                let key = parts.next().unwrap();
                let value = parts.next().unwrap_or("");
                if attrs.get(key).map_or(false, |v| v == value) {
                    return true;
                }
            }
        }

        false
    };

    // Remove the matched elements
    for node in document.descendants() {
        if should_block(&node.data()) {
            node.detach();
        }
    }

    // Write the modified HTML to the output file
    fs::write(&opt.output, document.to_string()).expect("Unable to write to output file");
}