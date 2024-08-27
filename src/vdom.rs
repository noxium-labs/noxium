use actix_web::{
    web, App, HttpServer, HttpResponse, Responder, Error, HttpRequest, HttpMessage,
    Result as ActixResult, middleware, dev::ServiceRequest, HttpServiceFactory
};
use actix_service::Service;
use askama::Template;
use serde::{Deserialize, Serialize};
use log::{info, error, debug};
use std::fs;
use std::sync::Arc;
use actix_web::middleware::Logger;
use actix_web::web::Json;
use actix_web::http::header::{X_REQUEST_ID, CONTENT_TYPE};
use std::env;
use sqlx::SqlitePool;
use actix_web::middleware::NormalizePath;
use actix_multipart::Multipart;
use std::io::Write;
use lazy_static::lazy_static;
use actix_web::http::header::HeaderValue;
use actix_service::Service as _;

// Virtual DOM implementation
#[derive(Debug, Clone)]
pub enum VNode {
    Element {
        tag: String,
        children: Vec<Rc<RefCell<VNode>>>,
        attributes: HashMap<String, String>,
        event_handlers: HashMap<String, Box<dyn Fn()>>,
    },
    Text(String),
    Fragment(Vec<Rc<RefCell<VNode>>>),
    Component {
        name: String,
        props: HashMap<String, String>,
        state: Rc<RefCell<dyn Any>>,
        component: Box<dyn Component>,
    },
}

#[derive(Debug, Clone)]
pub enum Patch {
    Replace(Rc<RefCell<VNode>>),
    Add(Rc<RefCell<VNode>>),
    Remove,
    UpdateAttributes(HashMap<String, Option<String>>),
    UpdateEventHandlers(HashMap<String, Box<dyn Fn()>>),
    UpdateState(String, Box<dyn Any>),
}

pub trait Component {
    fn render(&self) -> Rc<RefCell<VNode>>;
    fn component_did_mount(&mut self) {}
    fn component_will_unmount(&mut self) {}
}

impl VNode {
    pub fn new_element(tag: &str, attributes: HashMap<String, String>, children: Vec<Rc<RefCell<VNode>>>, event_handlers: HashMap<String, Box<dyn Fn()>>) -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(VNode::Element {
            tag: tag.to_string(),
            attributes,
            children,
            event_handlers,
        }))
    }

    pub fn new_text(text: &str) -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(VNode::Text(text.to_string())))
    }

    pub fn new_fragment(children: Vec<Rc<RefCell<VNode>>>) -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(VNode::Fragment(children)))
    }

    pub fn new_component(name: &str, props: HashMap<String, String>, state: Rc<RefCell<dyn Any>>, component: Box<dyn Component>) -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(VNode::Component {
            name: name.to_string(),
            props,
            state,
            component,
        }))
    }
}

pub fn diff(old: &Rc<RefCell<VNode>>, new: &Rc<RefCell<VNode>>) -> Vec<Patch> {
    let mut patches = Vec::new();
    
    match (&*old.borrow(), &*new.borrow()) {
        (VNode::Element { tag: old_tag, attributes: old_attrs, children: old_children, event_handlers: old_handlers },
         VNode::Element { tag: new_tag, attributes: new_attrs, children: new_children, event_handlers: new_handlers }) => {
            if old_tag != new_tag {
                patches.push(Patch::Replace(new.clone()));
            } else {
                let mut attrs_diff = HashMap::new();
                for (key, value) in new_attrs.iter() {
                    if let Some(old_value) = old_attrs.get(key) {
                        if old_value != value {
                            attrs_diff.insert(key.clone(), Some(value.clone()));
                        }
                    } else {
                        attrs_diff.insert(key.clone(), Some(value.clone()));
                    }
                }
                for (key, _) in old_attrs.iter() {
                    if !new_attrs.contains_key(key) {
                        attrs_diff.insert(key.clone(), None);
                    }
                }
                if !attrs_diff.is_empty() {
                    patches.push(Patch::UpdateAttributes(attrs_diff));
                }

                let mut handlers_diff = HashMap::new();
                for (event, handler) in new_handlers.iter() {
                    if let Some(old_handler) = old_handlers.get(event) {
                        if !std::ptr::eq(&**handler as *const _, &**old_handler as *const _) {
                            handlers_diff.insert(event.clone(), handler.clone());
                        }
                    } else {
                        handlers_diff.insert(event.clone(), handler.clone());
                    }
                }
                for event in old_handlers.keys() {
                    if !new_handlers.contains_key(event) {
                        handlers_diff.insert(event.clone(), Box::new(|| ()) as Box<dyn Fn()>);
                    }
                }
                if !handlers_diff.is_empty() {
                    patches.push(Patch::UpdateEventHandlers(handlers_diff));
                }

                let mut children_patches = Vec::new();
                let len = old_children.len().min(new_children.len());
                for i in 0..len {
                    children_patches.extend(diff(&old_children[i], &new_children[i]));
                }
                if old_children.len() > new_children.len() {
                    for i in new_children.len()..old_children.len() {
                        children_patches.push(Patch::Remove);
                    }
                } else if new_children.len() > old_children.len() {
                    for i in old_children.len()..new_children.len() {
                        children_patches.push(Patch::Add(new_children[i].clone()));
                    }
                }
                patches.extend(children_patches);
            }
        }
        (VNode::Text(old_text), VNode::Text(new_text)) => {
            if old_text != new_text {
                patches.push(Patch::Replace(new.clone()));
            }
        }
        (VNode::Fragment(old_children), VNode::Fragment(new_children)) => {
            let mut children_patches = Vec::new();
            let len = old_children.len().min(new_children.len());
            for i in 0..len {
                children_patches.extend(diff(&old_children[i], &new_children[i]));
            }
            if old_children.len() > new_children.len() {
                for i in new_children.len()..old_children.len() {
                    children_patches.push(Patch::Remove);
                }
            } else if new_children.len() > old_children.len() {
                for i in old_children.len()..new_children.len() {
                    children_patches.push(Patch::Add(new_children[i].clone()));
                }
            }
            patches.extend(children_patches);
        }
        (VNode::Component { name: old_name, props: old_props, state: old_state, component: old_component },
         VNode::Component { name: new_name, props: new_props, state: new_state, component: new_component }) => {
            if old_name != new_name {
                patches.push(Patch::Replace(new.clone()));
            } else {
                let mut state_diff = HashMap::new();
                if let Some(new_state) = new_state.borrow().downcast_ref::<String>() {
                    if let Some(old_state) = old_state.borrow().downcast_ref::<String>() {
                        if old_state != new_state {
                            state_diff.insert("state".to_string(), Box::new(new_state.clone()) as Box<dyn Any>);
                        }
                    } else {
                        state_diff.insert("state".to_string(), Box::new(new_state.clone()) as Box<dyn Any>);
                    }
                }
                if !state_diff.is_empty() {
                    patches.push(Patch::UpdateState("state".to_string(), Box::new(state_diff) as Box<dyn Any>));
                }
            }
        }
        _ => patches.push(Patch::Replace(new.clone())),
    }
    
    patches
}

impl fmt::Display for VNode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            VNode::Element { tag, children, attributes, .. } => {
                write!(f, "<{} ", tag)?;
                for (key, value) in attributes {
                    write!(f, "{}=\"{}\" ", key, value)?;
                }
                write!(f, ">")?;
                for child in children {
                    write!(f, "{}", child.borrow())?;
                }
                write!(f, "</{}>", tag)
            }
            VNode::Text(text) => write!(f, "{}", text),
            VNode::Fragment(children) => {
                for child in children {
                    write!(f, "{}", child.borrow())?;
                }
                Ok(())
            }
            VNode::Component { name, props, state, .. } => {
                write!(f, "<Component name=\"{}\" props=\"{:?}\" state=\"{:?}\"/>", name, props, state.borrow())
            }
        }
    }
}

pub fn apply_patches(root: &mut VNode, patches: &[Patch]) {
    let root = match root {
        VNode::Element { children, .. } => children,
        VNode::Fragment(children) => children,
        _ => return,
    };

    for patch in patches {
        match patch {
            Patch::Replace(new_node) => *root = vec![new_node.clone()],
            Patch::Add(node) => root.push(node.clone()),
            Patch::Remove => { root.pop(); },
            Patch::UpdateAttributes(attrs) => {
                if let VNode::Element { attributes, .. } = root.last_mut().unwrap().borrow_mut() {
                    for (key, value) in attrs {
                        match value {
                            Some(val) => attributes.insert(key.clone(), val.clone()),
                            None => attributes.remove(key),
                        };
                    }
                }
            }
            Patch::UpdateEventHandlers(handlers) => {
                if let VNode::Element { event_handlers, .. } = root.last_mut().unwrap().borrow_mut() {
                    for (event, handler) in handlers {
                        event_handlers.insert(event.clone(), handler.clone());
                    }
                }
            }
            Patch::UpdateState(key, state) => {
                if let VNode::Component { state: component_state, .. } = root.last_mut().unwrap().borrow_mut() {
                    if let Some(state) = state.downcast_ref::<String>() {
                        component_state.replace_with(|_| state.clone());
                    }
                }
            }
        }
    }
}

// Define a struct that represents our template data
#[derive(Template)]
#[template(path = "index.html")]
struct IndexTemplate {
    message: String,
}

// Define a struct for configuration data
#[derive(Deserialize, Serialize)]
struct Config {
    port: u16,
    database_url: String,
}

// Define a struct for user registration
#[derive(Deserialize, Serialize)]
struct UserRegistration {
    username: String,
    password: String,
}

// Define a struct for user details
#[derive(Serialize)]
struct UserDetails {
    id: u32,
    username: String,
}

// Define a custom error type for API errors
#[derive(Debug)]
enum ApiError {
    InvalidInput(String),
    InternalError(String),
    DatabaseError(String),
    AuthenticationError(String),
}

impl std::fmt::Display for ApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for ApiError {}

struct RateLimiter {
    requests: Arc<std::sync::Mutex<std::collections::HashMap<String, usize>>>,
}

async fn rate_limiter(req: ServiceRequest, srv: &actix_service::Service) -> Result<HttpResponse, Error> {
    let client_ip = req.connection_info().realip().unwrap_or("unknown").to_string();
    let mut state = req.app_data::<web::Data<RateLimiter>>().unwrap().requests.lock().unwrap();
    
    let counter = state.entry(client_ip.clone()).or_insert(0);
    *counter += 1;
    
    if *counter > 100 {
        return Ok(req.error_response(HttpResponse::TooManyRequests()));
    }

    Ok(srv.call(req).await?)
}

lazy_static! {
    static ref DB_POOL: Arc<SqlitePool> = Arc::new(SqlitePool::connect(&env::var("DATABASE_URL").unwrap()).unwrap());
}

async fn index() -> HttpResponse {
    let template = IndexTemplate {
        message: "Hello from the server!".to_string(),
    };

    let rendered = match template.render() {
        Ok(content) => content,
        Err(err) => {
            error!("Error rendering template: {}", err);
            return HttpResponse::InternalServerError().finish();
        }
    };

    HttpResponse::Ok()
        .content_type("text/html")
        .body(rendered)
}

async fn api_handler(req: HttpRequest, body: Json<Config>) -> ActixResult<HttpResponse> {
    let config = body.into_inner();

    info!("Received API request with port: {}", config.port);

    if config.port == 0 {
        return Err(ApiError::InvalidInput("Port cannot be zero".into()).into());
    }

    Ok(HttpResponse::Ok()
        .content_type("application/json")
        .json(config))
}

async fn upload_file(mut payload: Multipart) -> ActixResult<HttpResponse> {
    while let Some(item) = payload.next().await {
        let mut field = item?;
        let filename = field.filename().to_string();
        let filepath = format!("./uploads/{}", filename);

        let mut file = std::fs::File::create(filepath)?;
        while let Some(chunk) = field.next().await {
            let data = chunk?;
            file.write_all(&data)?;
        }
    }

    Ok(HttpResponse::Ok().body("File uploaded successfully"))
}

async fn get_data_from_db() -> ActixResult<HttpResponse> {
    let pool = DB_POOL.clone();
    let rows = sqlx::query!("SELECT id, name FROM items")
        .fetch_all(&*pool)
        .await
        .map_err(|e| ApiError::DatabaseError(e.to_string()))?;

    let result: Vec<_> = rows.into_iter().map(|row| (row.id, row.name)).collect();
    Ok(HttpResponse::Ok().json(result))
}

async fn log_request(req: ServiceRequest, srv: &actix_service::Service) -> Result<HttpResponse, Error> {
    debug!("Received request: {} {}", req.method(), req.uri());
    Ok(srv.call(req).await?)
}

async fn add_custom_headers(req: ServiceRequest, srv: &actix_service::Service) -> Result<HttpResponse, Error> {
    let mut res = srv.call(req).await?;
    res.headers_mut().insert(
        X_REQUEST_ID,
        HeaderValue::from_static("12345"),
    );
    Ok(res)
}

async fn handle_cors(req: ServiceRequest, srv: &actix_service::Service) -> Result<HttpResponse, Error> {
    let mut res = srv.call(req).await?;
    res.headers_mut().insert(
        actix_web::http::header::ACCESS_CONTROL_ALLOW_ORIGIN,
        HeaderValue::from_static("*"),
    );
    Ok(res)
}

fn read_config_from_file(file_path: &str) -> Result<Config, std::io::Error> {
    let content = fs::read_to_string(file_path)?;
    let config: Config = serde_json::from_str(&content)?;
    Ok(config)
}

async fn shutdown_signal() {
    tokio::signal::ctrl_c().await.expect("Failed to listen for shutdown signal");
    info!("Received shutdown signal, shutting down gracefully.");
}

// Mock user authentication function
fn authenticate_user(username: &str, password: &str) -> bool {
    username == "admin" && password == "password"
}

// Handler for user registration
async fn register_user(body: Json<UserRegistration>) -> ActixResult<HttpResponse> {
    let user = body.into_inner();

    if authenticate_user(&user.username, &user.password) {
        Ok(HttpResponse::Ok().body("User registered successfully"))
    } else {
        Err(ApiError::AuthenticationError("Invalid credentials".into()).into())
    }
}

// Handler for getting user details
async fn get_user_details(user_id: web::Path<u32>) -> ActixResult<HttpResponse> {
    let id = user_id.into_inner();
    
    // Mock user details
    let user = UserDetails {
        id,
        username: "admin".to_string(),
    };

    Ok(HttpResponse::Ok().json(user))
}

// Handler for serving static files
async fn static_file_handler(req: HttpRequest) -> ActixResult<HttpResponse> {
    let filename = req.match_info().get("filename").unwrap_or("index.html");
    let filepath = format!("./public/{}", filename);

    match fs::read_to_string(&filepath) {
        Ok(content) => Ok(HttpResponse::Ok().content_type("text/html").body(content)),
        Err(_) => Ok(HttpResponse::NotFound().body("File not found")),
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();

    let port = env::var("PORT").unwrap_or_else(|_| "8080".to_string()).parse::<u16>().unwrap();
    let database_url = env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite://:memory:".to_string());

    let pool = SqlitePool::connect(&database_url).await.unwrap();
    let pool = Arc::new(pool);
    DB_POOL = pool;

    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .wrap_fn(log_request)
            .wrap_fn(add_custom_headers)
            .wrap_fn(handle_cors)
            .wrap_fn(rate_limiter)
            .service(web::resource("/").route(web::get().to(index)))
            .service(web::resource("/api").route(web::post().to(api_handler)))
            .service(web::resource("/upload").route(web::post().to(upload_file)))
            .service(web::resource("/data").route(web::get().to(get_data_from_db)))
            .service(web::resource("/register").route(web::post().to(register_user)))
            .service(web::resource("/user/{user_id}").route(web::get().to(get_user_details)))
            .service(web::resource("/static/{filename:.*}").route(web::get().to(static_file_handler)))
            .default_service(web::route().to(|| HttpResponse::NotFound()))
            .service(
                web::resource("/status")
                    .route(web::get().to(|| HttpResponse::Ok().body("Server is running.")))
            )
            .wrap(NormalizePath::default())
    })
    .bind(format!("127.0.0.1:{}", port))?
    .run()
    .await
}