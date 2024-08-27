use actix_web::{web, App, HttpServer, HttpResponse, Responder, middleware};
use std::sync::{Arc, Mutex};
use tiberius::{Client, Config, AuthMethod};
use tokio::net::TcpStream;
use tokio_util::compat::TokioAsyncWriteCompatExt;
use futures::stream::StreamExt;

struct AppState {
    client: Mutex<Client<TcpStream>>,
    allowed_tables: Mutex<Vec<String>>,
}

async fn get_user(data: web::Data<Arc<AppState>>, id: web::Path<i32>) -> impl Responder {
    let mut client = data.client.lock().unwrap();
    let allowed_tables = data.allowed_tables.lock().unwrap();

    if !allowed_tables.contains(&"users".to_string()) {
        return HttpResponse::Forbidden().body("Access denied");
    }

    let query = format!("SELECT name FROM users WHERE id = {}", id);
    let result = client.simple_query(query).await;

    match result {
        Ok(mut row) => {
            if let Some(row) = row.next().await.unwrap() {
                let name: &str = row.get(0).unwrap();
                HttpResponse::Ok().body(format!("User: {}", name))
            } else {
                HttpResponse::NotFound().body("User not found")
            }
        },
        Err(_) => HttpResponse::InternalServerError().body("Error querying the database"),
    }
}

async fn set_user(data: web::Data<Arc<AppState>>, info: web::Json<(i32, String)>) -> impl Responder {
    let mut client = data.client.lock().unwrap();
    let (id, name) = info.into_inner();

    let query = format!("INSERT INTO users (id, name) VALUES ({}, '{}')", id, name);
    let result = client.simple_query(query).await;

    match result {
        Ok(_) => HttpResponse::Created().body("User added"),
        Err(_) => HttpResponse::InternalServerError().body("Error inserting into the database"),
    }
}

async fn update_user(data: web::Data<Arc<AppState>>, info: web::Json<(i32, String)>) -> impl Responder {
    let mut client = data.client.lock().unwrap();
    let (id, name) = info.into_inner();

    let query = format!("UPDATE users SET name = '{}' WHERE id = {}", name, id);
    let result = client.simple_query(query).await;

    match result {
        Ok(_) => HttpResponse::Ok().body("User updated"),
        Err(_) => HttpResponse::InternalServerError().body("Error updating the database"),
    }
}

async fn delete_user(data: web::Data<Arc<AppState>>, id: web::Path<i32>) -> impl Responder {
    let mut client = data.client.lock().unwrap();

    let query = format!("DELETE FROM users WHERE id = {}", id);
    let result = client.simple_query(query).await;

    match result {
        Ok(_) => HttpResponse::Ok().body("User deleted"),
        Err(_) => HttpResponse::InternalServerError().body("Error deleting from the database"),
    }
}

async fn list_users(data: web::Data<Arc<AppState>>) -> impl Responder {
    let mut client = data.client.lock().unwrap();
    let allowed_tables = data.allowed_tables.lock().unwrap();

    if !allowed_tables.contains(&"users".to_string()) {
        return HttpResponse::Forbidden().body("Access denied");
    }

    let query = "SELECT id, name FROM users";
    let result = client.simple_query(query).await;

    match result {
        Ok(mut rows) => {
            let mut response = String::new();
            while let Some(row) = rows.next().await.unwrap() {
                let id: i32 = row.get(0).unwrap();
                let name: &str = row.get(1).unwrap();
                response.push_str(&format!("ID: {}, Name: {}\n", id, name));
            }
            HttpResponse::Ok().body(response)
        },
        Err(_) => HttpResponse::InternalServerError().body("Error querying the database"),
    }
}

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    let mut config = Config::new();
    config.host("127.0.0.1");
    config.port(1433);
    config.authentication(AuthMethod::sql_server("SA", "your_password"));

    let tcp = TcpStream::connect(config.get_addr()).await.unwrap();
    tcp.set_nodelay(true).unwrap();
    let client = Client::connect(config, tcp.compat_write()).await.unwrap();

    let data = web::Data::new(Arc::new(AppState {
        client: Mutex::new(client),
        allowed_tables: Mutex::new(vec!["users".to_string()]),
    }));

    HttpServer::new(move || {
        App::new()
            .app_data(data.clone())
            .wrap(middleware::Logger::default())
            .wrap(middleware::Compress::default())
            .service(web::resource("/user/{id}").route(web::get().to(get_user)))
            .service(web::resource("/user").route(web::post().to(set_user)))
            .service(web::resource("/user/update").route(web::put().to(update_user)))
            .service(web::resource("/user/delete/{id}").route(web::delete().to(delete_user)))
            .service(web::resource("/users").route(web::get().to(list_users)))
    })
    .bind("127.0.0.1:5500")?
    .run()
    .await
}