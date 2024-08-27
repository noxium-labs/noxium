use warp::{Filter, Rejection, Reply};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use log::{info, error};
use validator::{Validate, ValidationErrors};
use regex::Regex;
use std::convert::Infallible;
use thiserror::Error;
use sqlx::SqlitePool;
use dotenv::dotenv;
use bcrypt::{hash, verify};
use std::env;

// Define a struct for a simple JSON response
#[derive(Debug, Serialize, Deserialize)]
struct Hello {
    message: String,
}

// Define a struct for request validation
#[derive(Debug, Deserialize, Validate)]
struct EchoRequest {
    #[validate(length(min = 1))]
    message: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct LoginRequest {
    username: String,
    password: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct AuthResponse {
    token: String,
}

// Custom error type for detailed error responses
#[derive(Debug, Error)]
pub enum AppError {
    #[error("Validation error")]
    ValidationError(#[from] ValidationErrors),
    #[error("Database error")]
    DatabaseError(#[from] sqlx::Error),
    #[error("Authentication error")]
    AuthError,
    #[error("Internal server error")]
    InternalError,
}

// Create a warp filter that handles GET requests to the root path
async fn hello() -> Result<impl Reply, Rejection> {
    Ok(warp::reply::json(&Hello {
        message: "Hello, World!".to_string(),
    }))
}

// Create a warp filter that handles POST requests to the "/echo" path
async fn echo(body: EchoRequest) -> Result<impl Reply, Rejection> {
    if let Err(e) = body.validate() {
        return Err(warp::reject::custom(AppError::ValidationError(e)));
    }
    Ok(warp::reply::json(&Hello {
        message: body.message,
    }))
}

// Simulate a database query
async fn get_user_from_db(username: &str) -> Result<Option<(String, String)>, AppError> {
    let pool = SqlitePool::connect("sqlite:./test.db").await?;
    let row: (String, String) = sqlx::query_as("SELECT username, password FROM users WHERE username = ?")
        .bind(username)
        .fetch_one(&pool)
        .await
        .ok();
    Ok(row)
}

// Handle user login
async fn login(body: LoginRequest) -> Result<impl Reply, Rejection> {
    let (stored_username, stored_password) = match get_user_from_db(&body.username).await {
        Ok(Some(row)) => row,
        Ok(None) => return Err(warp::reject::custom(AppError::AuthError)),
        Err(_) => return Err(warp::reject::custom(AppError::InternalError)),
    };

    if verify(&body.password, &stored_password).unwrap_or(false) {
        let token = "mock-token"; // Replace with real token generation
        Ok(warp::reply::json(&AuthResponse { token: token.to_string() }))
    } else {
        Err(warp::reject::custom(AppError::AuthError))
    }
}

// Middleware for logging requests
async fn log_request<F>(req: warp::filters::BoxedFilter<(impl Reply,)>, name: &str) -> impl Filter<Extract = (impl Reply,), Error = Rejection> + Clone
where
    F: warp::Filter + Clone + Send + Sync + 'static,
    F::Extract: warp::Reply,
{
    warp::log::custom(move |info| {
        info!(target: "warp", "{} - {} - {}", name, info.method(), info.path());
    })
    .and(req)
}

// Custom error handler
async fn handle_rejection(err: Rejection) -> Result<impl Reply, Infallible> {
    if let Some(e) = err.find::<AppError>() {
        match e {
            AppError::ValidationError(_) => Ok(warp::reply::with_status(
                "Validation error occurred",
                warp::http::StatusCode::BAD_REQUEST,
            )),
            AppError::DatabaseError(_) => Ok(warp::reply::with_status(
                "Database error occurred",
                warp::http::StatusCode::INTERNAL_SERVER_ERROR,
            )),
            AppError::AuthError => Ok(warp::reply::with_status(
                "Authentication error",
                warp::http::StatusCode::UNAUTHORIZED,
            )),
            AppError::InternalError => Ok(warp::reply::with_status(
                "Internal server error",
                warp::http::StatusCode::INTERNAL_SERVER_ERROR,
            )),
        }
    } else {
        error!("Unhandled rejection: {:?}", err);
        Ok(warp::reply::with_status(
            "Internal server error",
            warp::http::StatusCode::INTERNAL_SERVER_ERROR,
        ))
    }
}

// Define a struct for configuration
#[derive(Debug, Deserialize)]
struct Config {
    port: u16,
}

// Load configuration from environment variables or default
fn load_config() -> Config {
    dotenv().ok();
    let port = env::var("PORT")
        .unwrap_or_else(|_| "3030".to_string())
        .parse()
        .unwrap_or(3030);
    Config { port }
}

// Create a new route for /info that provides server information
async fn info_route() -> Result<impl Reply, Rejection> {
    Ok(warp::reply::json(&Hello {
        message: "Server Info: Rust Warp Server".to_string(),
    }))
}

// Health check endpoint
async fn health_check() -> Result<impl Reply, Rejection> {
    Ok(warp::reply::with_status("OK", warp::http::StatusCode::OK))
}

#[tokio::main]
async fn main() {
    // Initialize logging
    env_logger::init();

    // Load configuration
    let config = load_config();

    // Define the routes
    let hello_route = warp::path::end().and_then(hello);
    let echo_route = warp::path("echo")
        .and(warp::post())
        .and(warp::body::json())
        .and_then(echo);
    let login_route = warp::path("login")
        .and(warp::post())
        .and(warp::body::json())
        .and_then(login);
    let info_route = warp::path("info").and_then(info_route);
    let health_route = warp::path("health").and_then(health_check);

    // Combine the routes into a single filter with logging
    let routes = warp::get()
        .and(log_request(hello_route.boxed(), "GET /"))
        .or(warp::post().and(log_request(echo_route.boxed(), "POST /echo")))
        .or(warp::post().and(log_request(login_route.boxed(), "POST /login")))
        .or(log_request(info_route.boxed(), "GET /info"))
        .or(log_request(health_route.boxed(), "GET /health"));

    // Define the address to bind to
    let addr = SocketAddr::from(([127, 0, 0, 1], config.port));

    // Start the warp server
    info!("Server running on http://{}", addr);
    warp::serve(routes.with(warp::reject::custom(handle_rejection))).run(addr).await;
}