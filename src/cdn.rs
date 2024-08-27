use hyper::{Body, Request, Response, Server, Method, StatusCode};
use hyper::service::{make_service_fn, service_fn};
use hyper::header::{CONTENT_TYPE, CONTENT_ENCODING, CACHE_CONTROL, AUTHORIZATION};
use hyper_rustls::HttpsConnectorBuilder;
use tokio::fs::{File, read_dir};
use tokio::io::AsyncReadExt;
use tokio::sync::Mutex;
use std::convert::Infallible;
use std::collections::HashMap;
use std::sync::Arc;
use std::path::PathBuf;
use std::time::{SystemTime, Duration};
use mime_guess::from_path;
use futures::future::{BoxFuture, FutureExt};
use log::{info, warn, error};
use env_logger;
use rustls::{Certificate, PrivateKey, ServerConfig};
use std::fs;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct Config {
    rate_limit: u32,
    cache_duration: u64,
    auth_username: String,
    auth_password: String,
}

struct CacheEntry {
    data: Vec<u8>,
    last_access: SystemTime,
    content_type: String,
    encoding: Option<String>,
}

type Cache = Arc<Mutex<HashMap<String, CacheEntry>>>;
type RateLimiter = Arc<Mutex<HashMap<String, (u32, SystemTime)>>>;

async fn serve_file(req: Request<Body>, cache: Cache, rate_limiter: RateLimiter, config: Arc<Config>) -> Result<Response<Body>, Infallible> {
    let client_ip = req.headers().get("x-forwarded-for")
        .and_then(|ip| ip.to_str().ok())
        .unwrap_or("unknown");

    if !rate_limit(client_ip, rate_limiter.clone(), config.rate_limit).await {
        return Ok(Response::builder()
            .status(429)
            .body(Body::from("Too Many Requests"))
            .unwrap());
    }

    if !authorize(&req, &config) {
        return Ok(Response::builder()
            .status(StatusCode::UNAUTHORIZED)
            .header("WWW-Authenticate", "Basic realm=\"User Visible Realm\"")
            .body(Body::from("Unauthorized"))
            .unwrap());
    }

    let path = format!(".{}", req.uri().path());
    let path = PathBuf::from(path);

    let cache_key = req.uri().path().to_string();
    {
        let mut cache = cache.lock().await;
        if let Some(entry) = cache.get(&cache_key) {
            if entry.last_access.elapsed().unwrap() < Duration::new(config.cache_duration, 0) {
                info!("Serving from cache: {}", cache_key);
                let mut builder = Response::builder()
                    .header(CONTENT_TYPE, entry.content_type.clone())
                    .header(CACHE_CONTROL, "max-age=31536000");
                if let Some(encoding) = &entry.encoding {
                    builder = builder.header(CONTENT_ENCODING, encoding.clone());
                }
                return Ok(builder.body(Body::from(entry.data.clone())).unwrap());
            }
        }
    }

    let mut response = if path.is_file() {
        match File::open(&path).await {
            Ok(mut file) => {
                let mut buf = Vec::new();
                file.read_to_end(&mut buf).await.unwrap();

                let mime_type = from_path(&path).first_or_octet_stream();
                let compressed = compress_if_needed(&buf, mime_type.essence_str());

                {
                    let mut cache = cache.lock().await;
                    cache.insert(
                        cache_key.clone(),
                        CacheEntry {
                            data: compressed.clone(),
                            last_access: SystemTime::now(),
                            content_type: mime_type.to_string(),
                            encoding: Some("gzip".to_string()),
                        },
                    );
                }

                Response::builder()
                    .header(CONTENT_TYPE, mime_type.as_ref())
                    .header(CONTENT_ENCODING, "gzip")
                    .header(CACHE_CONTROL, "max-age=31536000")
                    .body(Body::from(compressed))
                    .unwrap()
            },
            Err(_) => not_found_response("File not found"),
        }
    } else if path.is_dir() {
        match serve_directory(&path).await {
            Ok(body) => Response::builder()
                .header(CONTENT_TYPE, "text/html")
                .body(Body::from(body))
                .unwrap(),
            Err(_) => not_found_response("Directory listing failed"),
        }
    } else {
        not_found_response("File not found")
    };

    Ok(response)
}

fn not_found_response(message: &str) -> Response<Body> {
    Response::builder()
        .status(404)
        .body(Body::from(message))
        .unwrap()
}

async fn serve_directory(path: &PathBuf) -> Result<String, std::io::Error> {
    let mut entries = read_dir(path).await?;
    let mut list = String::from("<html><body><ul>");

    while let Some(entry) = entries.next_entry().await? {
        let entry_name = entry.file_name().into_string().unwrap();
        list.push_str(&format!("<li><a href=\"{0}\">{0}</a></li>", entry_name));
    }

    list.push_str("</ul></body></html>");
    Ok(list)
}

fn compress_if_needed(data: &[u8], mime_type: &str) -> Vec<u8> {
    if mime_type.starts_with("text/") || mime_type == "application/javascript" {
        let mut encoder = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::default());
        encoder.write_all(data).unwrap();
        encoder.finish().unwrap()
    } else {
        data.to_vec()
    }
}

async fn rate_limit(ip: &str, rate_limiter: RateLimiter, max_requests: u32) -> bool {
    let mut limiter = rate_limiter.lock().await;
    let entry = limiter.entry(ip.to_string()).or_insert((0, SystemTime::now()));

    if entry.0 > max_requests && entry.1.elapsed().unwrap() < Duration::new(60, 0) {
        warn!("Rate limit exceeded for IP: {}", ip);
        return false;
    } else if entry.1.elapsed().unwrap() >= Duration::new(60, 0) {
        *entry = (1, SystemTime::now());
    } else {
        entry.0 += 1;
    }

    true
}

fn authorize(req: &Request<Body>, config: &Config) -> bool {
    if let Some(auth) = req.headers().get(AUTHORIZATION) {
        if let Ok(auth_str) = auth.to_str() {
            let encoded = base64::encode(format!("{}:{}", config.auth_username, config.auth_password));
            return auth_str == format!("Basic {}", encoded);
        }
    }
    false
}

fn tls_config(cert_path: &str, key_path: &str) -> ServerConfig {
    let certs = load_certs(cert_path);
    let key = load_private_key(key_path);
    let mut config = ServerConfig::new(rustls::NoClientAuth::new());
    config.set_single_cert(certs, key).unwrap();
    config
}

fn load_certs(path: &str) -> Vec<Certificate> {
    let certfile = fs::File::open(path).unwrap();
    let mut reader = std::io::BufReader::new(certfile);
    rustls_pemfile::certs(&mut reader).unwrap().into_iter().map(Certificate).collect()
}

fn load_private_key(path: &str) -> PrivateKey {
    let keyfile = fs::File::open(path).unwrap();
    let mut reader = std::io::BufReader::new(keyfile);
    let keys = rustls_pemfile::pkcs8_private_keys(&mut reader).unwrap();
    PrivateKey(keys[0].clone())
}

#[tokio::main]
async fn main() {
    env_logger::init();

    let config = Arc::new(Config {
        rate_limit: std::env::var("RATE_LIMIT").unwrap_or("100".to_string()).parse().unwrap(),
        cache_duration: std::env::var("CACHE_DURATION").unwrap_or("600".to_string()).parse().unwrap(),
        auth_username: std::env::var("AUTH_USERNAME").unwrap_or("user".to_string()),
        auth_password: std::env::var("AUTH_PASSWORD").unwrap_or("pass".to_string()),
    });

    let cache: Cache = Arc::new(Mutex::new(HashMap::new()));
    let rate_limiter: RateLimiter = Arc::new(Mutex::new(HashMap::new()));

    let addr = ([127, 0, 0, 1], 443).into();
    let cert_path = "cert.pem";
    let key_path = "key.pem";
    let tls_cfg = tls_config(cert_path, key_path);

    let https = HttpsConnectorBuilder::new()
        .with_tls_config(tls_cfg)
        .https_only()
        .enable_http1()
        .build();

    let make_svc = make_service_fn(|_| {
        let cache = cache.clone();
        let rate_limiter = rate_limiter.clone();
        let config = config.clone();

        async move {
            Ok::<_, Infallible>(service_fn(move |req| {
                serve_file(req, cache.clone(), rate_limiter.clone(), config.clone())
            }))
        }
    });

    let server = Server::builder(https)
        .serve(make_svc);

    if let Err(e) = server.await {
        error!("server error: {}", e);
    }
}