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