use warp::{Filter, Rejection, Reply};
use serde::{Deserialize, Serialize};
use serde_json::json;
use jsonwebtoken::{encode, Header, EncodingKey};
use std::collections::HashMap;

#[derive(Serialize, Deserialize)]
struct Claims {
    sub: String,
    exp: usize,
}

#[tokio::main]
async fn main() {
    let static_files = warp::path("static").and(warp::fs::dir("static"));

    let frontend1 = warp::path("frontend1").and(warp::fs::file("frontend1/index.html"));
    let frontend2 = warp::path("frontend2").and(warp::fs::file("frontend2/index.html"));

    let auth = warp::path("auth")
        .and(warp::post())
        .and(warp::body::json())
        .and_then(authenticate);

    let api = warp::path("api")
        .and(warp::get())
        .and_then(api_handler);

    let routes = static_files.or(frontend1).or(frontend2).or(auth).or(api);

    warp::serve(routes).run(([127, 0, 0, 1], 3030)).await;
}

async fn authenticate(credentials: HashMap<String, String>) -> Result<impl Reply, Rejection> {
    let username = credentials.get("username").unwrap_or(&"".to_string()).clone();
    let password = credentials.get("password").unwrap_or(&"".to_string()).clone();

    if username == "user" && password == "password" {
        let claims = Claims {
            sub: username,
            exp: 10000000000,
        };
        let token = encode(&Header::default(), &claims, &EncodingKey::secret("secret".as_ref())).unwrap();
        Ok(warp::reply::json(&json!({ "token": token })))
    } else {
        Ok(warp::reply::with_status("Unauthorized", warp::http::StatusCode::UNAUTHORIZED))
    }
}

async fn api_handler() -> Result<impl Reply, Rejection> {
    Ok(warp::reply::json(&json!({ "message": "Hello from the API!" })))
}