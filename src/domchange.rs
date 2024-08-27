use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{
    Document, Element, HtmlElement, HtmlInputElement, HtmlButtonElement, HtmlTextAreaElement,
    HtmlSelectElement, HtmlOptionElement, HtmlDivElement, HtmlSpanElement, HtmlTableElement,
    HtmlTableRowElement, HtmlTableCellElement, HtmlFormElement, HtmlAnchorElement, HtmlImageElement,
    HtmlListElement, HtmlListItemElement, HtmlCanvasElement, HtmlVideoElement
};

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = document)]
    pub fn get_element_by_id(id: &str) -> Option<Element>;

    #[wasm_bindgen(js_namespace = document)]
    pub fn create_element(tag_name: &str) -> Result<Element, JsValue>;

    #[wasm_bindgen(js_namespace = document, js_name = querySelector)]
    pub fn query_selector(selector: &str) -> Option<Element>;
}

#[wasm_bindgen]
pub fn manipulate_dom() {
    // Create a container div
    let container = create_element("div").unwrap();
    container.set_id("container");
    container.set_attribute("style", "padding: 20px; border: 2px solid #ccc; border-radius: 10px; background-color: #f9f9f9;").unwrap();

    // Create and style a header element
    let header = create_element("h1").unwrap();
    header.set_inner_html("Extensive DOM Manipulation Example");
    header.set_attribute("style", "color: #333; text-align: center;").unwrap();
    container.append_child(&header).unwrap();

    // Create a styled paragraph
    let paragraph = create_element("p").unwrap();
    paragraph.set_inner_html("This example showcases a comprehensive range of HTML elements and interactions.");
    paragraph.set_attribute("style", "font-size: 18px; color: #555; margin-bottom: 20px;").unwrap();
    container.append_child(&paragraph).unwrap();

    // Create an input element
    let input = create_element("input").unwrap();
    input.set_attribute("type", "text").unwrap();
    input.set_attribute("placeholder", "Enter text here...").unwrap();
    input.set_id("input-text");
    input.set_attribute("style", "padding: 10px; border-radius: 5px; border: 1px solid #ddd; width: 100%;").unwrap();
    container.append_child(&input).unwrap();

    // Create a text area
    let textarea = create_element("textarea").unwrap();
    textarea.set_attribute("placeholder", "Enter more information...").unwrap();
    textarea.set_id("textarea-info");
    textarea.set_attribute("rows", "4").unwrap();
    textarea.set_attribute("cols", "50").unwrap();
    textarea.set_attribute("style", "padding: 10px; border-radius: 5px; border: 1px solid #ddd; width: 100%; margin-top: 10px;").unwrap();
    container.append_child(&textarea).unwrap();

    // Create a select dropdown with multiple options
    let select = create_element("select").unwrap();
    select.set_id("dropdown-select");
    let options = vec!["Select an option", "Option 1", "Option 2", "Option 3"];
    for option_text in options {
        let option = create_element("option").unwrap();
        option.set_attribute("value", option_text).unwrap();
        option.set_inner_html(option_text);
        select.append_child(&option).unwrap();
    }
    select.set_attribute("style", "padding: 10px; border-radius: 5px; border: 1px solid #ddd; width: 100%; margin-top: 10px;").unwrap();
    container.append_child(&select).unwrap();

    // Create a button element
    let button = create_element("button").unwrap();
    button.set_inner_html("Submit");
    button.set_id("submit-button");
    button.set_attribute("style", "padding: 10px 20px; background-color: #007bff; color: white; border: none; border-radius: 5px; cursor: pointer; margin-top: 20px;").unwrap();
    container.append_child(&button).unwrap();

    // Create a div to display results
    let result_div = create_element("div").unwrap();
    result_div.set_id("result-div");
    result_div.set_attribute("style", "margin-top: 20px; padding: 10px; border: 1px solid #ddd; background-color: #fff; border-radius: 5px;").unwrap();
    container.append_child(&result_div).unwrap();

    // Create a form with various inputs
    let form = create_element("form").unwrap();
    form.set_id("form-example");
    form.set_attribute("style", "margin-top: 30px; padding: 20px; border: 1px solid #ddd; border-radius: 5px; background-color: #e9ecef;").unwrap();
    
    let name_label = create_element("label").unwrap();
    name_label.set_inner_html("Name:");
    name_label.set_attribute("for", "form-name").unwrap();
    form.append_child(&name_label).unwrap();
    
    let name_input = create_element("input").unwrap();
    name_input.set_id("form-name");
    name_input.set_attribute("type", "text").unwrap();
    name_input.set_attribute("placeholder", "Enter your name").unwrap();
    name_input.set_attribute("style", "padding: 10px; margin-bottom: 10px; border-radius: 5px; border: 1px solid #ddd; width: 100%;").unwrap();
    form.append_child(&name_input).unwrap();

    let email_label = create_element("label").unwrap();
    email_label.set_inner_html("Email:");
    email_label.set_attribute("for", "form-email").unwrap();
    form.append_child(&email_label).unwrap();
    
    let email_input = create_element("input").unwrap();
    email_input.set_id("form-email");
    email_input.set_attribute("type", "email").unwrap();
    email_input.set_attribute("placeholder", "Enter your email").unwrap();
    email_input.set_attribute("style", "padding: 10px; margin-bottom: 10px; border-radius: 5px; border: 1px solid #ddd; width: 100%;").unwrap();
    form.append_child(&email_input).unwrap();

    let submit_form_button = create_element("button").unwrap();
    submit_form_button.set_inner_html("Submit Form");
    submit_form_button.set_attribute("type", "submit").unwrap();
    submit_form_button.set_attribute("style", "padding: 10px 20px; background-color: #28a745; color: white; border: none; border-radius: 5px; cursor: pointer;").unwrap();
    form.append_child(&submit_form_button).unwrap();
    container.append_child(&form).unwrap();

    // Create a table with data
    let table = create_element("table").unwrap();
    table.set_id("data-table");
    table.set_attribute("style", "margin-top: 30px; border-collapse: collapse; width: 100%;").unwrap();

    let thead = create_element("thead").unwrap();
    let header_row = create_element("tr").unwrap();
    let headers = vec!["Header 1", "Header 2", "Header 3"];
    for header_text in headers {
        let th = create_element("th").unwrap();
        th.set_inner_html(header_text);
        th.set_attribute("style", "border: 1px solid #ddd; padding: 8px;").unwrap();
        header_row.append_child(&th).unwrap();
    }
    thead.append_child(&header_row).unwrap();
    table.append_child(&thead).unwrap();

    let tbody = create_element("tbody").unwrap();
    for i in 1..=3 {
        let row = create_element("tr").unwrap();
        for j in 1..=3 {
            let cell = create_element("td").unwrap();
            cell.set_inner_html(&format!("Row {} Cell {}", i, j));
            cell.set_attribute("style", "border: 1px solid #ddd; padding: 8px;").unwrap();
            row.append_child(&cell).unwrap();
        }
        tbody.append_child(&row).unwrap();
    }
    table.append_child(&tbody).unwrap();
    container.append_child(&table).unwrap();

    // Create a list of items
    let ul = create_element("ul").unwrap();
    ul.set_id("item-list");
    ul.set_attribute("style", "margin-top: 30px; padding: 0; list-style-type: disc;").unwrap();

    let list_items = vec!["Item 1", "Item 2", "Item 3"];
    for item_text in list_items {
        let li = create_element("li").unwrap();
        li.set_inner_html(item_text);
        li.set_attribute("style", "padding: 5px; border-bottom: 1px solid #ddd;").unwrap();
        ul.append_child(&li).unwrap();
    }
    container.append_child(&ul).unwrap();

    // Create an anchor element
    let anchor = create_element("a").unwrap();
    anchor.set_attribute("href", "https://www.example.com").unwrap();
    anchor.set_attribute("style", "display: block; margin-top: 20px; color: #007bff; text-decoration: none;").unwrap();
    anchor.set_inner_html("Go to Example.com");
    container.append_child(&anchor).unwrap();

    // Create an image element
    let image = create_element("img").unwrap();
    image.set_attribute("src", "https://via.placeholder.com/150").unwrap();
    image.set_attribute("alt", "Placeholder Image").unwrap();
    image.set_attribute("style", "display: block; margin-top: 20px; border: 1px solid #ddd; border-radius: 5px;").unwrap();
    container.append_child(&image).unwrap();

    // Create a canvas element
    let canvas = create_element("canvas").unwrap();
    canvas.set_id("drawing-canvas");
    canvas.set_attribute("width", "200").unwrap();
    canvas.set_attribute("height", "100").unwrap();
    canvas.set_attribute("style", "border: 1px solid #ddd; margin-top: 20px;").unwrap();
    container.append_child(&canvas).unwrap();

    // Draw on the canvas
    let canvas = canvas.dyn_into::<HtmlCanvasElement>().unwrap();
    let context = canvas.get_context("2d").unwrap().unwrap().dyn_into::<web_sys::CanvasRenderingContext2d>().unwrap();
    context.set_fill_style(&JsValue::from_str("lightblue"));
    context.fill_rect(10.0, 10.0, 150.0, 80.0);

    // Create a video element
    let video = create_element("video").unwrap();
    video.set_attribute("width", "320").unwrap();
    video.set_attribute("height", "240").unwrap();
    video.set_attribute("controls", "true").unwrap();
    video.set_attribute("style", "display: block; margin-top: 20px; border: 1px solid #ddd; border-radius: 5px;").unwrap();

    let source = create_element("source").unwrap();
    source.set_attribute("src", "https://www.w3schools.com/html/mov_bbb.mp4").unwrap();
    source.set_attribute("type", "video/mp4").unwrap();

    video.append_child(&source).unwrap();
    container.append_child(&video).unwrap();

    // Add event listeners
    let button = get_element_by_id("submit-button").unwrap();
    let button = button.dyn_into::<HtmlButtonElement>().unwrap();
    let button_closure = Closure::wrap(Box::new(move || {
        let input = get_element_by_id("input-text").unwrap();
        let input = input.dyn_into::<HtmlInputElement>().unwrap();
        let textarea = get_element_by_id("textarea-info").unwrap();
        let textarea = textarea.dyn_into::<HtmlTextAreaElement>().unwrap();
        let select = get_element_by_id("dropdown-select").unwrap();
        let select = select.dyn_into::<HtmlSelectElement>().unwrap();
        let result_div = get_element_by_id("result-div").unwrap();
        let result_div = result_div.dyn_into::<HtmlElement>().unwrap();

        let result_text = format!(
            "<strong>Input:</strong> {}<br><strong>Textarea:</strong> {}<br><strong>Select:</strong> {}",
            input.value(),
            textarea.value(),
            select.value()
        );
        result_div.set_inner_html(&result_text);
    }) as Box<dyn Fn()>);

    button.add_event_listener_with_callback("click", button_closure.as_ref().unchecked_ref()).unwrap();
    button_closure.forget();

    let form = get_element_by_id("form-example").unwrap();
    let form = form.dyn_into::<HtmlFormElement>().unwrap();
    let form_closure = Closure::wrap(Box::new(move || {
        let name = get_element_by_id("form-name").unwrap().dyn_into::<HtmlInputElement>().unwrap().value();
        let email = get_element_by_id("form-email").unwrap().dyn_into::<HtmlInputElement>().unwrap().value();
        let result_div = get_element_by_id("result-div").unwrap();
        let result_div = result_div.dyn_into::<HtmlElement>().unwrap();

        let form_result_text = format!(
            "<strong>Name:</strong> {}<br><strong>Email:</strong> {}",
            name, email
        );
        result_div.set_inner_html(&form_result_text);
    }) as Box<dyn Fn()>);

    form.add_event_listener_with_callback("submit", form_closure.as_ref().unchecked_ref()).unwrap();
    form_closure.forget();

    // Append the container to the body
    let body = get_element_by_id("body").unwrap();
    body.append_child(&container).unwrap();
}