use redis::{Client, Commands, RedisResult};
use actix_web::{web, App, HttpServer, HttpResponse, Responder};
use std::sync::{Arc, Mutex};
use serde::Deserialize;

#[derive(Deserialize)]
struct KeyValue {
    key: String,
    value: String,
}

#[derive(Deserialize)]
struct Expiration {
    key: String,
    expiration: u64,
}

#[derive(Deserialize)]
struct AllowedKey {
    key: String,
}

struct AppState {
    redis_client: Mutex<Client>,
    allowed_keys: Mutex<Vec<String>>,
}

async fn get_value(data: web::Data<Arc<AppState>>, key: web::Path<String>) -> impl Responder {
    let client = data.redis_client.lock().unwrap();
    let allowed_keys = data.allowed_keys.lock().unwrap();
    
    if !allowed_keys.contains(&key.into_inner()) {
        return HttpResponse::Forbidden().body("Access denied");
    }

    let mut con = client.get_connection().unwrap();
    let value: RedisResult<String> = con.get(&*key);
    match value {
        Ok(val) => HttpResponse::Ok().body(val),
        Err(_) => HttpResponse::NotFound().body("Key not found"),
    }
}

async fn set_value(data: web::Data<Arc<AppState>>, info: web::Json<KeyValue>) -> impl Responder {
    let client = data.redis_client.lock().unwrap();
    let KeyValue { key, value } = info.into_inner();

    let mut con = client.get_connection().unwrap();
    let _: RedisResult<()> = con.set(&key, value);

    HttpResponse::Ok().body("Value set")
}

async fn set_expiration(data: web::Data<Arc<AppState>>, info: web::Json<Expiration>) -> impl Responder {
    let client = data.redis_client.lock().unwrap();
    let Expiration { key, expiration } = info.into_inner();

    let mut con = client.get_connection().unwrap();
    let _: RedisResult<()> = con.set_ex(&key, "dummy_value", expiration);

    HttpResponse::Ok().body("Expiration set")
}

async fn delete_key(data: web::Data<Arc<AppState>>, key: web::Path<String>) -> impl Responder {
    let client = data.redis_client.lock().unwrap();

    let mut con = client.get_connection().unwrap();
    let _: RedisResult<()> = con.del(&*key);

    HttpResponse::Ok().body("Key deleted")
}

async fn list_keys(data: web::Data<Arc<AppState>>) -> impl Responder {
    let client = data.redis_client.lock().unwrap();

    let mut con = client.get_connection().unwrap();
    let keys: RedisResult<Vec<String>> = con.keys("*");

    match keys {
        Ok(key_list) => HttpResponse::Ok().json(key_list),
        Err(_) => HttpResponse::InternalServerError().body("Failed to list keys"),
    }
}

async fn update_allowed_keys(data: web::Data<Arc<AppState>>, key: web::Json<AllowedKey>) -> impl Responder {
    let mut allowed_keys = data.allowed_keys.lock().unwrap();
    allowed_keys.push(key.key.clone());

    HttpResponse::Ok().body("Allowed keys updated")
}

async fn ping_redis(data: web::Data<Arc<AppState>>) -> impl Responder {
    let client = data.redis_client.lock().unwrap();

    let mut con = client.get_connection().unwrap();
    let pong: RedisResult<String> = con.ping();
    match pong {
        Ok(_) => HttpResponse::Ok().body("Pong"),
        Err(_) => HttpResponse::InternalServerError().body("Failed to ping Redis"),
    }
}

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    let redis_client = Client::open("redis://127.0.0.1/").unwrap();
    let data = web::Data::new(Arc::new(AppState {
        redis_client: Mutex::new(redis_client),
        allowed_keys: Mutex::new(vec!["allowed_key".to_string()]),
    }));

    HttpServer::new(move || {
        App::new()
            .app_data(data.clone())
            .wrap(actix_web::middleware::Logger::default())
            .wrap(actix_web::middleware::Compress::default())
            .service(web::resource("/get/{key}").to(get_value))
            .service(web::resource("/set").route(web::post().to(set_value)))
            .service(web::resource("/set_expiration").route(web::post().to(set_expiration)))
            .service(web::resource("/delete/{key}").route(web::delete().to(delete_key)))
            .service(web::resource("/list_keys").route(web::get().to(list_keys)))
            .service(web::resource("/update_allowed_keys").route(web::post().to(update_allowed_keys)))
            .service(web::resource("/ping").route(web::get().to(ping_redis)))
    })
    .bind("127.0.0.1:5500")?
    .run()
    .await
}