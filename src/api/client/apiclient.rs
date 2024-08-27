use reqwest::{Client, Error, Response, StatusCode};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;
use tokio::time::sleep;
use log::{info, warn, error};
use config::{Config, File, Environment};
use std::fmt;

#[derive(Serialize, Deserialize, Debug)]
struct ApiResponse {
    data: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct AppConfig {
    api_base_url: String,
    api_key: String,
    timeout: u64,
    retry_attempts: u32,
    retry_delay: u64,
}

#[derive(Debug)]
enum ApiClientError {
    RequestFailed(StatusCode),
    Unauthorized,
    Timeout,
    TooManyRequests,
    Unexpected(String),
}

impl fmt::Display for ApiClientError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ApiClientError::RequestFailed(code) => write!(f, "Request failed with status code: {}", code),
            ApiClientError::Unauthorized => write!(f, "Unauthorized access"),
            ApiClientError::Timeout => write!(f, "Request timed out"),
            ApiClientError::TooManyRequests => write!(f, "Too many requests"),
            ApiClientError::Unexpected(err) => write!(f, "Unexpected error: {}", err),
        }
    }
}

impl std::error::Error for ApiClientError {}

async fn handle_response(response: Response) -> Result<ApiResponse, ApiClientError> {
    let status = response.status();
    match status {
        StatusCode::OK => {
            let json_response = response.json::<ApiResponse>().await.map_err(|e| ApiClientError::Unexpected(e.to_string()))?;
            Ok(json_response)
        }
        StatusCode::UNAUTHORIZED => {
            error!("Unauthorized access - check your API key or credentials");
            Err(ApiClientError::Unauthorized)
        }
        StatusCode::TOO_MANY_REQUESTS => {
            warn!("Too many requests - consider increasing retry delay");
            Err(ApiClientError::TooManyRequests)
        }
        _ => {
            error!("Unexpected server response: {:?}", status);
            Err(ApiClientError::RequestFailed(status))
        }
    }
}

async fn get_request(client: &Client, url: &str, headers: Option<HashMap<String, String>>, query_params: Option<HashMap<&str, &str>>) -> Result<ApiResponse, ApiClientError> {
    let mut request = client.get(url);

    if let Some(h) = headers {
        request = request.headers(h.into_iter().map(|(k, v)| (k.parse().unwrap(), v.parse().unwrap())).collect());
    }

    if let Some(params) = query_params {
        request = request.query(&params);
    }

    let response = request.send().await.map_err(|e| ApiClientError::Unexpected(e.to_string()))?;
    handle_response(response).await
}

async fn post_request(client: &Client, url: &str, headers: Option<HashMap<String, String>>, payload: &ApiResponse) -> Result<ApiResponse, ApiClientError> {
    let mut request = client.post(url).json(payload);

    if let Some(h) = headers {
        request = request.headers(h.into_iter().map(|(k, v)| (k.parse().unwrap(), v.parse().unwrap())).collect());
    }

    let response = request.send().await.map_err(|e| ApiClientError::Unexpected(e.to_string()))?;
    handle_response(response).await
}

async fn request_with_retries<F>(config: &AppConfig, operation: F) -> Result<ApiResponse, ApiClientError>
where
    F: Fn() -> Result<ApiResponse, ApiClientError> + Copy,
{
    let mut attempts = config.retry_attempts;
    loop {
        match operation() {
            Ok(response) => return Ok(response),
            Err(e) => {
                if attempts == 0 {
                    error!("Failed after multiple retries: {:?}", e);
                    return Err(e);
                }
                match &e {
                    ApiClientError::TooManyRequests => {
                        warn!("Too many requests - backing off for {} seconds", config.retry_delay);
                        sleep(Duration::from_secs(config.retry_delay)).await;
                    }
                    ApiClientError::Timeout => {
                        error!("Request timed out. Retrying...");
                    }
                    _ => {
                        error!("Request failed. Retrying... Remaining attempts: {}", attempts);
                    }
                }
                attempts -= 1;
                sleep(Duration::from_secs(config.retry_delay)).await;
            }
        }
    }
}

fn load_config() -> Result<AppConfig, config::ConfigError> {
    let mut settings = Config::new();
    settings.merge(File::with_name("config"))?;
    settings.merge(Environment::with_prefix("APP"))?;
    settings.try_into()
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    let config = load_config()?;

    let client = Client::builder()
        .timeout(Duration::from_secs(config.timeout))
        .build().map_err(|e| ApiClientError::Unexpected(e.to_string()))?;

    let get_url = format!("{}/get-endpoint", config.api_base_url);
    let post_url = format!("{}/post-endpoint", config.api_base_url);
    
    let mut headers = HashMap::new();
    headers.insert("Authorization".to_string(), format!("Bearer {}", config.api_key));
    headers.insert("Custom-Header".to_string(), "value".to_string());

    let mut query_params = HashMap::new();
    query_params.insert("query_param1", "value1");
    query_params.insert("query_param2", "value2");

    let get_response = request_with_retries(&config, || {
        get_request(&client, &get_url, Some(headers.clone()), Some(query_params.clone()))
    }).await?;

    info!("GET Response: {:?}", get_response);

    let post_payload = ApiResponse { data: "Some JSON data".into() };

    let post_response = request_with_retries(&config, || {
        post_request(&client, &post_url, Some(headers.clone()), &post_payload)
    }).await?;

    info!("POST Response: {:?}", post_response);

    Ok(())
}