use actix_files::NamedFile;
use actix_web::{web, App, HttpServer, Result};

async fn index() -> Result<NamedFile> {
    NamedFile::open("./static/index.html") // Serve a basic HTML file initially
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .route("/", web::get().to(index))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}