use actix_session::{CookieSession, Session};
use actix_web::{web, App, HttpServer, HttpResponse, Responder, middleware, HttpRequest};
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};
use std::sync::Mutex;
use std::collections::HashMap;

// Struct for user information
#[derive(Serialize, Deserialize, Clone)]
struct User {
    username: String,
    last_login: u64,
    email: String,
}

// Struct for user registration
#[derive(Serialize, Deserialize)]
struct RegisterUser {
    username: String,
    email: String,
}

// Struct for updating user information
#[derive(Serialize, Deserialize)]
struct UpdateUser {
    email: Option<String>,
}

// Struct for deleting user
#[derive(Serialize, Deserialize)]
struct DeleteUser {
    username: String,
}

// Global state to keep track of registered users
struct AppState {
    users: Mutex<HashMap<String, User>>,
}

// Middleware for logging requests
async fn log_request(req: HttpRequest) -> impl Responder {
    println!("Incoming request: {} {}", req.method(), req.path());
    HttpResponse::Ok()
}

// Register a new user
async fn register_user(
    data: web::Data<AppState>,
    user: web::Json<RegisterUser>,
) -> impl Responder {
    let mut users = data.users.lock().unwrap();
    if users.contains_key(&user.username) {
        return HttpResponse::Conflict().json("User already exists");
    }

    let new_user = User {
        username: user.username.clone(),
        email: user.email.clone(),
        last_login: 0,
    };
    users.insert(user.username.clone(), new_user);

    HttpResponse::Ok().json("User registered successfully")
}

// Log in a user and set session data
async fn login(
    session: Session,
    data: web::Data<AppState>,
    user: web::Json<User>,
) -> impl Responder {
    let mut users = data.users.lock().unwrap();
    if let Some(mut stored_user) = users.get_mut(&user.username) {
        let login_time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        stored_user.last_login = login_time;
        session.insert("user", &stored_user).unwrap();
        HttpResponse::Ok().json("Login successful")
    } else {
        HttpResponse::Unauthorized().json("User not found")
    }
}

// Get session information
async fn get_session_info(session: Session) -> impl Responder {
    if let Some(user) = session.get::<User>("user").unwrap() {
        HttpResponse::Ok().json(user)
    } else {
        HttpResponse::Ok().json("No user logged in")
    }
}

// Update user information
async fn update_user(
    session: Session,
    data: web::Data<AppState>,
    update: web::Json<UpdateUser>,
) -> impl Responder {
    if let Some(mut user) = session.get::<User>("user").unwrap() {
        if let Some(email) = &update.email {
            user.email = email.clone();
        }

        let mut users = data.users.lock().unwrap();
        if let Some(stored_user) = users.get_mut(&user.username) {
            *stored_user = user.clone();
        }

        session.insert("user", &user).unwrap();
        HttpResponse::Ok().json("User updated successfully")
    } else {
        HttpResponse::Unauthorized().json("No user logged in")
    }
}

// Logout and clear session data
async fn logout(session: Session) -> impl Responder {
    session.clear();
    HttpResponse::Ok().json("Logged out successfully")
}

// Delete a user
async fn delete_user(
    data: web::Data<AppState>,
    delete: web::Json<DeleteUser>,
) -> impl Responder {
    let mut users = data.users.lock().unwrap();
    if users.remove(&delete.username).is_some() {
        HttpResponse::Ok().json("User deleted successfully")
    } else {
        HttpResponse::NotFound().json("User not found")
    }
}

// List all registered users
async fn list_users(data: web::Data<AppState>) -> impl Responder {
    let users = data.users.lock().unwrap();
    let user_list: Vec<User> = users.values().cloned().collect();
    HttpResponse::Ok().json(user_list)
}

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    let app_state = web::Data::new(AppState {
        users: Mutex::new(HashMap::new()),
    });

    HttpServer::new(move || {
        App::new()
            .app_data(app_state.clone())
            .wrap(middleware::Logger::default())
            .wrap(CookieSession::signed(&[0; 32]).secure(false))
            .route("/register", web::post().to(register_user))
            .route("/login", web::post().to(login))
            .route("/session", web::get().to(get_session_info))
            .route("/update", web::put().to(update_user))
            .route("/logout", web::post().to(logout))
            .route("/delete", web::delete().to(delete_user))
            .route("/users", web::get().to(list_users))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}