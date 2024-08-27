use warp::Filter;
use serde::{Serialize, Deserialize};
use uuid::Uuid;
use std::sync::{Arc, RwLock};
use std::collections::HashMap;

// Define the Item struct for our API
#[derive(Serialize, Deserialize, Clone)]
struct Item {
    id: Uuid,
    name: String,
}

// In-memory database to hold items
#[derive(Clone)]
struct Database {
    items: Arc<RwLock<HashMap<Uuid, Item>>>,
}

impl Database {
    fn new() -> Self {
        let mut items = HashMap::new();
        items.insert(Uuid::new_v4(), Item { id: Uuid::new_v4(), name: "Initial Item".to_string() });
        Database {
            items: Arc::new(RwLock::new(items)),
        }
    }

    fn get_items(&self) -> Vec<Item> {
        let items = self.items.read().unwrap();
        items.values().cloned().collect()
    }

    fn get_item(&self, id: Uuid) -> Option<Item> {
        let items = self.items.read().unwrap();
        items.get(&id).cloned()
    }

    fn add_item(&self, item: Item) {
        let mut items = self.items.write().unwrap();
        items.insert(item.id, item);
    }

    fn update_item(&self, id: Uuid, name: String) -> Result<(), &'static str> {
        let mut items = self.items.write().unwrap();
        if let Some(item) = items.get_mut(&id) {
            item.name = name;
            Ok(())
        } else {
            Err("Item not found")
        }
    }

    fn delete_item(&self, id: Uuid) -> Result<(), &'static str> {
        let mut items = self.items.write().unwrap();
        if items.remove(&id).is_some() {
            Ok(())
        } else {
            Err("Item not found")
        }
    }
}

// Create the warp filters for the API
#[tokio::main]
async fn main() {
    let db = Database::new();
    let db = Arc::new(db);

    // GET /items - Retrieve all items
    let get_items = warp::path("items")
        .and(warp::get())
        .and(with_db(db.clone()))
        .map(|db: Arc<Database>| {
            warp::reply::json(&db.get_items())
        });

    // GET /items/{id} - Retrieve a single item by ID
    let get_item = warp::path!("items" / Uuid)
        .and(warp::get())
        .and(with_db(db.clone()))
        .map(|id: Uuid, db: Arc<Database>| {
            match db.get_item(id) {
                Some(item) => warp::reply::json(&item),
                None => warp::reply::with_status("Item not found", warp::http::StatusCode::NOT_FOUND),
            }
        });

    // POST /items - Add a new item
    let post_item = warp::path("items")
        .and(warp::post())
        .and(warp::body::json())
        .and(with_db(db.clone()))
        .map(|item: Item, db: Arc<Database>| {
            db.add_item(item);
            warp::reply::with_status("Item added", warp::http::StatusCode::CREATED)
        });

    // PUT /items/{id} - Update an item by ID
    let put_item = warp::path!("items" / Uuid)
        .and(warp::put())
        .and(warp::body::json())
        .and(with_db(db.clone()))
        .map(|id: Uuid, name: String, db: Arc<Database>| {
            match db.update_item(id, name) {
                Ok(()) => warp::reply::with_status("Item updated", warp::http::StatusCode::OK),
                Err(e) => warp::reply::with_status(e, warp::http::StatusCode::NOT_FOUND),
            }
        });

    // DELETE /items/{id} - Delete an item by ID
    let delete_item = warp::path!("items" / Uuid)
        .and(warp::delete())
        .and(with_db(db.clone()))
        .map(|id: Uuid, db: Arc<Database>| {
            match db.delete_item(id) {
                Ok(()) => warp::reply::with_status("Item deleted", warp::http::StatusCode::OK),
                Err(e) => warp::reply::with_status(e, warp::http::StatusCode::NOT_FOUND),
            }
        });

    // Combine all routes into a single filter
    let routes = get_items
        .or(get_item)
        .or(post_item)
        .or(put_item)
        .or(delete_item);

    // Start the warp server
    warp::serve(routes).run(([127, 0, 0, 1], 3030)).await;
}

// Helper function to pass the database to the warp filters
fn with_db(db: Arc<Database>) -> impl Filter<Extract = (Arc<Database>,), Error = warp::Rejection> + Clone {
    warp::any().map(move || db.clone())
}