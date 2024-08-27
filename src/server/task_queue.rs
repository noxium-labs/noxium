use actix_web::{web, App, HttpServer, HttpResponse, Responder};
use redis::AsyncCommands;
use serde::{Serialize, Deserialize};
use tokio::task;
use tokio::net::TcpListener;
use uuid::Uuid;
use std::sync::Arc;

#[derive(Serialize, Deserialize)]
struct Task {
    id: String,
    status: String,
    port: Option<u16>,
}

// Function to process a task by starting a server on a dynamic port
async fn process_task(task_id: String, client: redis::Client) -> Result<(), redis::RedisError> {
    // Bind a new TcpListener to port 0 to get a dynamic port
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();

    // Create an asynchronous connection to Redis
    let mut con = client.get_async_connection().await?;
    
    // Update the task status to 'running' and store the assigned port in Redis
    con.hset(&task_id, "status", "running").await?;
    con.hset(&task_id, "port", port).await?;

    // Start a new Actix web server on the dynamic port
    let server = HttpServer::new(|| {
        App::new()
            .route("/", web::post().to(echo))  // Define a route for handling POST requests
    })
    .listen(listener)?  // Use the dynamically assigned listener
    .run();

    println!("Server started for task {} on port {}", task_id, port);

    // Run the server until it's manually stopped
    server.await?;

    // Update task status to 'completed' once the server stops
    con.hset(&task_id, "status", "completed").await?;

    Ok(())
}

// Echo handler that returns the received request body
async fn echo(req_body: String) -> impl Responder {
    HttpResponse::Ok().body(req_body)
}

// Handler to add a new task
async fn add_task() -> impl Responder {
    // Generate a new unique task ID
    let task_id = Uuid::new_v4().to_string();
    
    // Create a Redis client and establish a connection
    let client = redis::Client::open("redis://127.0.0.1/").unwrap();
    let mut con = client.get_async_connection().await.unwrap();

    // Create a new task in Redis with status 'pending'
    con.hset(&task_id, "status", "pending").await.unwrap();
    con.lpush("task_queue", &task_id).await.unwrap();

    // Spawn a new asynchronous task for processing
    let client_clone = client.clone();
    tokio::spawn(async move {
        if let Err(e) = process_task(task_id.clone(), client_clone).await {
            // Log an error if the task processing fails
            eprintln!("Error processing task {}: {:?}", task_id, e);
        }
    });

    // Respond with the newly created task's ID and initial status
    HttpResponse::Ok().json(Task {
        id: task_id,
        status: "pending".to_string(),
        port: None,
    })
}

// Handler to get the status of a task
async fn get_task_status(task_id: web::Path<String>) -> impl Responder {
    // Create a Redis client and establish a connection
    let client = redis::Client::open("redis://127.0.0.1/").unwrap();
    let mut con = client.get_async_connection().await.unwrap();
    
    // Retrieve the task status from Redis
    match con.hget::<_, _, String>(&task_id, "status").await {
        Ok(status) => {
            // If the task exists, get the assigned port if available
            let port: Option<u16> = con.hget(&task_id, "port").await.ok();
            HttpResponse::Ok().json(Task {
                id: task_id.to_string(),
                status,
                port,
            })
        },
        Err(_) => HttpResponse::NotFound().body("Task not found"),  // Return 404 if the task does not exist
    }
}

// Main function to start the Actix web server
#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    // Initialize and run the main Actix web server
    HttpServer::new(|| {
        App::new()
            .route("/add_task", web::post().to(add_task))  // Route to add a new task
            .route("/task/{task_id}", web::get().to(get_task_status))  // Route to get task status
    })
    .bind("127.0.0.1:5500")?  // Bind to the specified address and port
    .run()
    .await
}