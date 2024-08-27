use serde::{Serialize, Deserialize}; // Import Serde for serializing and deserializing data
use std::fs; // Import standard library filesystem module
use std::collections::HashMap; // Import HashMap for simulating DOM attributes

// Define a struct to represent a DOM element with attributes and children
#[derive(Serialize, Deserialize, Clone)]
struct DomElement {
    tag: String,                      // HTML tag of the element, e.g., "div", "p"
    attributes: HashMap<String, String>, // Key-value pairs for attributes, e.g., "id", "class"
    children: Vec<DomElement>,        // Nested elements or children of this DOM element
}

impl DomElement {
    // Method to create a new DOM element
    fn new(tag: &str) -> Self {
        DomElement {
            tag: tag.to_string(),
            attributes: HashMap::new(),
            children: Vec::new(),
        }
    }

    // Method to add an attribute to the DOM element
    fn set_attribute(&mut self, key: &str, value: &str) {
        self.attributes.insert(key.to_string(), value.to_string());
    }

    // Method to remove an attribute from the DOM element
    fn remove_attribute(&mut self, key: &str) {
        self.attributes.remove(key);
    }

    // Method to add a child element to this DOM element
    fn add_child(&mut self, child: DomElement) {
        self.children.push(child);
    }

    // Method to remove a child element by tag name
    fn remove_child_by_tag(&mut self, tag: &str) {
        self.children.retain(|child| child.tag != tag);
    }

    // Method to simulate rendering the DOM element as an HTML string
    fn render(&self) -> String {
        // Start with the opening tag and add attributes
        let mut html = format!("<{}", self.tag);
        for (key, value) in &self.attributes {
            html.push_str(&format!(" {}=\"{}\"", key, value));
        }
        html.push('>');

        // Recursively render child elements
        for child in &self.children {
            html.push_str(&child.render());
        }

        // Close the tag
        html.push_str(&format!("</{}>", self.tag));
        html
    }

    // Method to find a child element by tag name
    fn find_child_by_tag(&self, tag: &str) -> Option<&DomElement> {
        for child in &self.children {
            if child.tag == tag {
                return Some(child);
            }
        }
        None
    }

    // Method to replace a child element by tag name
    fn replace_child_by_tag(&mut self, tag: &str, new_child: DomElement) {
        for child in &mut self.children {
            if child.tag == tag {
                *child = new_child;
                return;
            }
        }
    }

    // Method to count the number of elements with a specific tag
    fn count_elements_by_tag(&self, tag: &str) -> usize {
        let mut count = 0;
        for child in &self.children {
            if child.tag == tag {
                count += 1;
            }
            count += child.count_elements_by_tag(tag);
        }
        count
    }

    // Method to update the text content of an element (simulated with a "text" tag)
    fn update_text_content(&mut self, new_text: &str) {
        if self.tag == "text" {
            self.children.clear(); // Remove existing text nodes
            let new_text_node = DomElement::new("text");
            self.add_child(new_text_node);
        } else {
            for child in &mut self.children {
                child.update_text_content(new_text);
            }
        }
    }

    // Method to set or update styles directly in the style attribute
    fn set_style(&mut self, style: &str) {
        self.set_attribute("style", style);
    }

    // Method to simulate cloning a DOM element
    fn clone_element(&self) -> DomElement {
        self.clone()
    }

    // Method to simulate adding an event listener (e.g., "click" event)
    fn add_event_listener(&mut self, event: &str, handler: &str) {
        self.set_attribute(&format!("on{}", event), handler);
    }
}

fn main() {
    // Load the file to simulate working with DOM nodes from an HTML file
    let path = "./static/index.html";

    // Check if the file exists
    if fs::metadata(path).is_ok() {
        println!("Found static file: {}", path);

        // Simulate creating a DOM element from the HTML file
        let mut body = DomElement::new("body"); // Create a <body> element

        // Add some attributes to the body
        body.set_attribute("id", "main-body");
        body.set_attribute("class", "container");

        // Create a child element, e.g., a <div> inside the body
        let mut div = DomElement::new("div");
        div.set_attribute("class", "content");

        // Create a nested child element, e.g., a <p> inside the <div>
        let mut paragraph = DomElement::new("p");
        paragraph.set_attribute("class", "text");
        paragraph.set_attribute("style", "color: blue;");

        // Add text content to the paragraph (simulated as a text node)
        let text_node = DomElement {
            tag: "text".to_string(), // Simulate a text node with tag "text"
            attributes: HashMap::new(),
            children: Vec::new(),
        };
        paragraph.add_child(text_node); // Add the text node as a child

        // Add the paragraph to the div
        div.add_child(paragraph);

        // Add the div to the body
        body.add_child(div);

        // Create more complex DOM structure with additional elements
        let mut header = DomElement::new("header");
        header.set_attribute("id", "main-header");
        header.set_attribute("class", "header");

        let mut nav = DomElement::new("nav");
        nav.set_attribute("class", "navigation");

        let mut ul = DomElement::new("ul");
        ul.set_attribute("class", "menu");

        let mut li1 = DomElement::new("li");
        li1.set_attribute("class", "menu-item");

        let mut a1 = DomElement::new("a");
        a1.set_attribute("href", "#");
        a1.set_attribute("class", "menu-link");
        let link_text1 = DomElement::new("text");
        a1.add_child(link_text1);
        li1.add_child(a1);

        let mut li2 = DomElement::new("li");
        li2.set_attribute("class", "menu-item");

        let mut a2 = DomElement::new("a");
        a2.set_attribute("href", "#");
        a2.set_attribute("class", "menu-link");
        let link_text2 = DomElement::new("text");
        a2.add_child(link_text2);
        li2.add_child(a2);

        // Add list items to the unordered list
        ul.add_child(li1);
        ul.add_child(li2);

        // Add the unordered list to the nav
        nav.add_child(ul);

        // Add the nav to the header
        header.add_child(nav);

        // Add the header to the body
        body.add_child(header);

        // Render the DOM to an HTML string and print it
        let rendered_html = body.render();
        println!("Rendered HTML:\n{}", rendered_html);

        // Modify some attributes and elements
        body.set_attribute("style", "background-color: lightgrey;");
        div.set_attribute("style", "padding: 20px;");

        // Remove the class attribute from the paragraph
        paragraph.remove_attribute("class");

        // Find a specific child element
        if let Some(found_child) = body.find_child_by_tag("header") {
            println!("Found child with tag 'header': {:?}", found_child.tag);
        } else {
            println!("Child with tag 'header' not found.");
        }

        // Replace a child element
        let new_div = DomElement::new("div");
        body.replace_child_by_tag("div", new_div);

        // Count the number of specific elements
        let num_paragraphs = body.count_elements_by_tag("p");
        println!("Number of <p> elements: {}", num_paragraphs);

        // Update text content of an element
        if let Some(text_element) = body.find_child_by_tag("text") {
            let mut text_element_clone = text_element.clone_element();
            text_element_clone.update_text_content("New Text Content");
            println!("Updated text content.");
        }

        // Add event listeners
        body.add_event_listener("click", "handleClick()");
        div.add_event_listener("mouseover", "handleMouseOver()");

        // Create and add more elements for demonstration
        let mut footer = DomElement::new("footer");
        footer.set_attribute("id", "main-footer");
        footer.set_attribute("class", "footer");

        let mut contact_div = DomElement::new("div");
        contact_div.set_attribute("class", "contact");

        let mut address = DomElement::new("address");
        address.set_attribute("class", "address-info");

        let address_text = DomElement::new("text");
        address_text.add_child(DomElement {
            tag: "text".to_string(),
            attributes: HashMap::new(),
            children: vec![],
        });
        address.add_child(address_text);

        contact_div.add_child(address);
        footer.add_child(contact_div);
        body.add_child(footer);

        // Render the updated DOM to an HTML string and print it
        let updated_html = body.render();
        println!("Updated HTML:\n{}", updated_html);

        // Perform more manipulations and checks
        let num_footers = body.count_elements_by_tag("footer");
        println!("Number of <footer> elements: {}", num_footers);

        // Remove an element
        body.remove_child_by_tag("header");
        println!("Removed <header> element.");

        // Render the final DOM to an HTML string and print it
        let final_html = body.render();
        println!("Final HTML:\n{}", final_html);
    } else {
        println!("Static file not found: {}", path);
    }
}