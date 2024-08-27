use redis::{Client, Commands, RedisResult};
use actix_web::{web, App, HttpServer, HttpResponse, Responder, middleware};
use std::sync::{Arc, Mutex};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;
use actix_web::middleware::Logger;

#[derive(Debug, Deserialize, Serialize)]
struct KeyValue {
    key: String,
    value: String,
}

struct AppState {
    redis_client: Mutex<Client>,
    allowed_keys: Mutex<HashMap<String, bool>>,
    request_timeout: Duration,
}

async fn read_data(data: web::Data<Arc<AppState>>, key: web::Path<String>) -> impl Responder {
    let client = data.redis_client.lock().unwrap();
    let allowed_keys = data.allowed_keys.lock().unwrap();

    if !allowed_keys.contains_key(&key.into_inner()) {
        return HttpResponse::Forbidden().body("Access denied");
    }

    let mut con = client.get_connection().unwrap();
    let value: RedisResult<String> = con.get(&*key);
    match value {
        Ok(val) => HttpResponse::Ok().body(val),
        Err(_) => HttpResponse::NotFound().body("Key not found"),
    }
}

async fn write_data(data: web::Data<Arc<AppState>>, info: web::Json<KeyValue>) -> impl Responder {
    let client = data.redis_client.lock().unwrap();
    let KeyValue { key, value } = info.into_inner();

    let mut con = client.get_connection().unwrap();
    let _: RedisResult<()> = con.set(&key, value);

    HttpResponse::Ok().body("Data written")
}

async fn delete_data(data: web::Data<Arc<AppState>>, key: web::Path<String>) -> impl Responder {
    let client = data.redis_client.lock().unwrap();
    let mut con = client.get_connection().unwrap();
    let result: RedisResult<()> = con.del(&*key);

    match result {
        Ok(_) => HttpResponse::Ok().body("Data deleted"),
        Err(_) => HttpResponse::InternalServerError().body("Error deleting data"),
    }
}

async fn list_keys(data: web::Data<Arc<AppState>>) -> impl Responder {
    let client = data.redis_client.lock().unwrap();
    let mut con = client.get_connection().unwrap();
    let keys: RedisResult<Vec<String>> = con.keys("*");
    
    match keys {
        Ok(key_list) => HttpResponse::Ok().json(key_list),
        Err(_) => HttpResponse::InternalServerError().body("Error retrieving keys"),
    }
}

async fn bulk_write_data(data: web::Data<Arc<AppState>>, info: web::Json<Vec<KeyValue>>) -> impl Responder {
    let client = data.redis_client.lock().unwrap();
    let mut con = client.get_connection().unwrap();

    for KeyValue { key, value } in info.into_inner() {
        let _: RedisResult<()> = con.set(&key, value);
    }

    HttpResponse::Ok().body("Bulk data written")
}

async fn check_key_existence(data: web::Data<Arc<AppState>>, key: web::Path<String>) -> impl Responder {
    let client = data.redis_client.lock().unwrap();
    let mut con = client.get_connection().unwrap();
    let exists: RedisResult<bool> = con.exists(&*key);

    match exists {
        Ok(true) => HttpResponse::Ok().body("Key exists"),
        Ok(false) => HttpResponse::NotFound().body("Key does not exist"),
        Err(_) => HttpResponse::InternalServerError().body("Error checking key existence"),
    }
}

async fn set_allowed_keys(data: web::Data<Arc<AppState>>, keys: web::Json<Vec<String>>) -> impl Responder {
    let mut allowed_keys = data.allowed_keys.lock().unwrap();
    allowed_keys.clear();
    for key in keys.into_inner() {
        allowed_keys.insert(key, true);
    }

    HttpResponse::Ok().body("Allowed keys updated")
}

async fn get_allowed_keys(data: web::Data<Arc<AppState>>) -> impl Responder {
    let allowed_keys = data.allowed_keys.lock().unwrap();
    let keys: Vec<String> = allowed_keys.keys().cloned().collect();

    HttpResponse::Ok().json(keys)
}

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    let redis_client = Client::open("redis://127.0.0.1/").unwrap();
    let data = web::Data::new(Arc::new(AppState {
        redis_client: Mutex::new(redis_client),
        allowed_keys: Mutex::new(HashMap::new()),
        request_timeout: Duration::from_secs(5),
    }));

    HttpServer::new(move || {
        App::new()
            .app_data(data.clone())
            .wrap(Logger::default())
            .wrap(middleware::Compress::default())
            .service(web::resource("/read/{key}").to(read_data))
            .service(web::resource("/write").route(web::post().to(write_data)))
            .service(web::resource("/delete/{key}").route(web::delete().to(delete_data)))
            .service(web::resource("/keys").route(web::get().to(list_keys)))
            .service(web::resource("/bulk_write").route(web::post().to(bulk_write_data)))
            .service(web::resource("/check/{key}").route(web::get().to(check_key_existence)))
            .service(web::resource("/allowed_keys").route(web::post().to(set_allowed_keys)))
            .service(web::resource("/allowed_keys").route(web::get().to(get_allowed_keys)))
    })
    .bind("127.0.0.1:5500")?
    .run()
    .await
}