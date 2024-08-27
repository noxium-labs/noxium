use warp::{Filter, Rejection, Reply};
use jsonwebtoken::{encode, decode, Header, EncodingKey, DecodingKey, Validation, TokenData};
use serde::{Deserialize, Serialize};
use chrono::{Utc, Duration};
use std::env;
use ratelimit::RateLimiter;

// Define a struct to represent JWT claims
#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: String,
    exp: usize,
    roles: Vec<String>,
    permissions: Vec<String>,
}

// Define a struct for refresh token claims
#[derive(Debug, Serialize, Deserialize)]
struct RefreshTokenClaims {
    sub: String,
    exp: usize,
}

// Define custom authentication errors
#[derive(Debug)]
enum AuthError {
    InvalidToken,
    ExpiredToken,
    Unauthorized,
    Forbidden,
    RateLimited,
    InvalidRefreshToken,
}

// Implement the Reject trait for custom errors
impl warp::reject::Reject for AuthError {}

// Function to authenticate a JWT token
async fn authenticate(token: Option<String>) -> Result<TokenData<Claims>, Rejection> {
    let secret = env::var("JWT_SECRET").expect("JWT_SECRET must be set");
    let decoding_key = DecodingKey::from_secret(secret.as_ref());
    let validation = Validation::default();

    match token {
        Some(t) => match decode::<Claims>(&t, &decoding_key, &validation) {
            Ok(token_data) => {
                if token_data.claims.exp > (Utc::now().timestamp() as usize) {
                    Ok(token_data)
                } else {
                    Err(warp::reject::custom(AuthError::ExpiredToken))
                }
            },
            Err(_) => Err(warp::reject::custom(AuthError::InvalidToken)),
        },
        None => Err(warp::reject::custom(AuthError::Unauthorized)),
    }
}

// Function to generate a JWT token
fn generate_token(user: &str, roles: Vec<String>, permissions: Vec<String>) -> String {
    let secret = env::var("JWT_SECRET").expect("JWT_SECRET must be set");
    let expiration = (Utc::now() + Duration::hours(1)).timestamp() as usize;
    let claims = Claims {
        sub: user.to_string(),
        exp: expiration,
        roles,
        permissions,
    };
    let encoding_key = EncodingKey::from_secret(secret.as_ref());
    encode(&Header::default(), &claims, &encoding_key).expect("Failed to generate token")
}

// Function to generate a refresh token
fn generate_refresh_token(user: &str) -> String {
    let secret = env::var("REFRESH_TOKEN_SECRET").expect("REFRESH_TOKEN_SECRET must be set");
    let expiration = (Utc::now() + Duration::days(30)).timestamp() as usize;
    let claims = RefreshTokenClaims {
        sub: user.to_string(),
        exp: expiration,
    };
    let encoding_key = EncodingKey::from_secret(secret.as_ref());
    encode(&Header::default(), &claims, &encoding_key).expect("Failed to generate refresh token")
}

// Function to authenticate a refresh token
async fn authenticate_refresh_token(token: Option<String>) -> Result<TokenData<RefreshTokenClaims>, Rejection> {
    let secret = env::var("REFRESH_TOKEN_SECRET").expect("REFRESH_TOKEN_SECRET must be set");
    let decoding_key = DecodingKey::from_secret(secret.as_ref());
    let validation = Validation::default();

    match token {
        Some(t) => match decode::<RefreshTokenClaims>(&t, &decoding_key, &validation) {
            Ok(token_data) => {
                if token_data.claims.exp > (Utc::now().timestamp() as usize) {
                    Ok(token_data)
                } else {
                    Err(warp::reject::custom(AuthError::InvalidRefreshToken))
                }
            },
            Err(_) => Err(warp::reject::custom(AuthError::InvalidRefreshToken)),
        },
        None => Err(warp::reject::custom(AuthError::Unauthorized)),
    }
}

// Middleware function to check authentication and roles
fn with_auth(required_role: Option<String>) -> impl Filter<Extract = (TokenData<Claims>,), Error = Rejection> + Clone {
    warp::header::optional("Authorization")
        .and_then(move |auth_header: Option<String>| {
            let required_role = required_role.clone();
            async move {
                let token_data = authenticate(auth_header).await?;
                if let Some(role) = &required_role {
                    if !token_data.claims.roles.contains(role) {
                        return Err(warp::reject::custom(AuthError::Forbidden));
                    }
                }
                Ok(token_data)
            }
        })
}

// Middleware function for rate limiting
fn rate_limit() -> impl Filter<Extract = (), Error = Rejection> + Clone {
    let limiter = RateLimiter::new(10, Duration::minutes(1));
    warp::any().map(move || limiter.check().map_err(|_| warp::reject::custom(AuthError::RateLimited)))
}

#[tokio::main]
async fn main() {
    let auth_filter = with_auth(Some("admin".to_string()));
    let rate_limit_filter = rate_limit();

    // Route to login and generate a token
    let login = warp::path("login")
        .and(warp::post())
        .and(warp::body::json())
        .map(|user: String| {
            let token = generate_token(&user, vec!["admin".to_string()], vec!["read".to_string(), "write".to_string()]);
            let refresh_token = generate_refresh_token(&user);
            warp::reply::json(&serde_json::json!({
                "token": token,
                "refresh_token": refresh_token,
            }))
        });

    // Route to refresh a token
    let refresh = warp::path("refresh")
        .and(warp::post())
        .and(warp::body::json())
        .map(|refresh_token: String| {
            let token_data = authenticate_refresh_token(Some(refresh_token)).await;
            match token_data {
                Ok(data) => {
                    let new_token = generate_token(&data.claims.sub, vec!["admin".to_string()], vec!["read".to_string(), "write".to_string()]);
                    warp::reply::json(&serde_json::json!({
                        "token": new_token,
                    }))
                },
                Err(_) => warp::reply::with_status("Invalid refresh token", warp::http::StatusCode::UNAUTHORIZED),
            }
        });

    // Route to a protected endpoint
    let protected = warp::path("protected")
        .and(rate_limit_filter)
        .and(auth_filter)
        .map(|token_data: TokenData<Claims>| {
            warp::reply::json(&token_data.claims)
        });

    // Combine routes
    let routes = login.or(refresh).or(protected);

    // Start the server on 127.0.0.1:3030
    warp::serve(routes).run(([127, 0, 0, 1], 3030)).await;
}