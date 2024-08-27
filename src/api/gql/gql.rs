use async_graphql::{Schema, Object, Context, FieldResult, EmptyMutation, EmptySubscription, Enum, ID, InputObject};
use async_graphql_actix_web::{GraphQLRequest, GraphQLResponse};
use actix_web::{web, App, HttpServer, HttpResponse, HttpRequest, Result as ActixResult};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use actix_service::Service;
use actix_web::middleware::Logger;

// Define a User struct for the GraphQL schema
#[derive(SimpleObject, Clone)]
struct User {
    id: ID,
    name: String,
    age: i32,
}

#[derive(InputObject)]
struct NewUser {
    name: String,
    age: i32,
}

#[derive(Default)]
struct Query;

#[Object]
impl Query {
    async fn hello(&self, ctx: &Context<'_>) -> FieldResult<String> {
        Ok("Hello, world!".to_string())
    }

    async fn get_user(&self, ctx: &Context<'_>, id: ID) -> FieldResult<User> {
        // Dummy data for example
        Ok(User {
            id,
            name: "John Doe".to_string(),
            age: 30,
        })
    }

    async fn list_users(&self, ctx: &Context<'_>) -> FieldResult<Vec<User>> {
        // Dummy data for example
        Ok(vec![
            User {
                id: ID::new("1"),
                name: "John Doe".to_string(),
                age: 30,
            },
            User {
                id: ID::new("2"),
                name: "Jane Smith".to_string(),
                age: 25,
            },
        ])
    }
}

#[derive(Default)]
struct Mutation;

#[Object]
impl Mutation {
    async fn create_user(&self, ctx: &Context<'_>, new_user: NewUser) -> FieldResult<User> {
        // Dummy data for example
        Ok(User {
            id: ID::new("1"),
            name: new_user.name,
            age: new_user.age,
        })
    }

    async fn update_user(&self, ctx: &Context<'_>, id: ID, new_name: String) -> FieldResult<User> {
        // Dummy data for example
        Ok(User {
            id,
            name: new_name,
            age: 30, // Assume age remains the same for simplicity
        })
    }

    async fn delete_user(&self, ctx: &Context<'_>, id: ID) -> FieldResult<String> {
        // Dummy data for example
        Ok(format!("User with ID {} deleted", id))
    }
}

type MySchema = Schema<Query, Mutation, EmptySubscription>;

// GraphQL handler
async fn graphql_handler(schema: web::Data<Arc<MySchema>>, req: GraphQLRequest) -> GraphQLResponse {
    schema.execute(req.into_inner()).await.into()
}

// REST API handler
async fn rest_api_handler(req: web::HttpRequest) -> HttpResponse {
    HttpResponse::Ok().json("REST API endpoint")
}

// Authentication middleware
async fn auth_middleware<S>(
    req: HttpRequest,
    srv: &S,
) -> ActixResult<HttpResponse>
where
    S: Service<Request = HttpRequest, Response = HttpResponse, Error = actix_service::ServiceError>,
{
    let auth_header = req.headers().get("Authorization");
    if auth_header.is_some() {
        srv.call(req).await
    } else {
        Ok(req.error_response(HttpResponse::Unauthorized()))
    }
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let schema = Arc::new(Schema::build(Query::default(), Mutation::default(), EmptySubscription)
        .finish());

    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .app_data(web::Data::new(schema.clone()))
            .service(web::resource("/graphql").guard(web::guard().post()).to(graphql_handler))
            .service(web::resource("/api").route(web::get().to(rest_api_handler)))
            .wrap_fn(auth_middleware) // Add authentication middleware
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}