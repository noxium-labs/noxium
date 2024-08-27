use wasmtime::{Engine, Linker, Module, Store, Instance, Val, Trap};
use std::fs::File;
use std::io::prelude::*;
use std::env;
use std::error::Error;
use log::{info, error};
use tokio::task;
use futures::future::join_all;
use hyper::{Body, Request, Response, Server, service::{make_service_fn, service_fn}};
use serde::{Deserialize, Serialize};
use serde_json::json;

/// Loads a WASM module from a file.
///
/// # Arguments
///
/// * `path` - The path to the WASM file.
///
/// # Returns
///
/// * `Result<Vec<u8>, Box<dyn Error>>` - Returns the module bytes or an error.
fn load_wasm_module(path: &str) -> Result<Vec<u8>, Box<dyn Error>> {
    info!("Loading WASM module from: {}", path);
    let mut file = File::open(path)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;
    Ok(buffer)
}

/// Creates and configures a Wasmtime instance from the WASM bytes.
///
/// # Arguments
///
/// * `wasm_bytes` - The byte code of the WASM module.
///
/// # Returns
///
/// * `Result<Instance, Box<dyn Error>>` - Returns the instance or an error.
fn create_wasm_instance(wasm_bytes: &[u8]) -> Result<Instance, Box<dyn Error>> {
    info!("Creating WASM instance");
    let engine = Engine::default();
    let store = Store::new(&engine);
    let module = Module::new(&engine, wasm_bytes)?;
    let mut linker = Linker::new(&engine);

    // Example configuration for linker
    // linker.func("env", "log", |s: &str| println!("{}", s))?;

    let instance = linker.instantiate(&store, &module)?;
    Ok(instance)
}

/// Executes a function from the WASM instance and processes the result.
///
/// # Arguments
///
/// * `instance` - The WASM instance.
/// * `func_name` - The name of the function to call.
///
/// # Returns
///
/// * `Result<String, Box<dyn Error>>` - Returns the result of the function or an error.
async fn execute_wasm_function(instance: &Instance, func_name: &str) -> Result<String, Box<dyn Error>> {
    info!("Executing function: {}", func_name);
    let func = instance.get_func(func_name)
        .ok_or_else(|| format!("Function '{}' not found in WASM module", func_name))?;
    
    let result = func.call(&[]).map_err(|trap| {
        error!("Execution error: {:?}", trap);
        Box::new(trap) as Box<dyn Error>
    })?;

    let mut output = String::new();
    for val in result {
        match val {
            Val::I32(i) => output.push_str(&format!("I32: {}\n", i)),
            Val::I64(i) => output.push_str(&format!("I64: {}\n", i)),
            Val::F32(f) => output.push_str(&format!("F32: {}\n", f)),
            Val::F64(f) => output.push_str(&format!("F64: {}\n", f)),
            _ => output.push_str("Other type\n"),
        }
    }

    Ok(output)
}

/// Runs multiple WASM modules in parallel.
///
/// # Arguments
///
/// * `paths` - A vector of paths to WASM modules.
/// * `func_name` - The function name to execute.
///
/// # Returns
///
/// * `Result<(), Box<dyn Error>>` - Returns `Ok(())` on success or an error.
async fn run_parallel_wasm_modules(paths: Vec<&str>, func_name: &str) -> Result<(), Box<dyn Error>> {
    let tasks: Vec<_> = paths.into_iter().map(|path| {
        task::spawn(async move {
            let wasm_bytes = match load_wasm_module(path) {
                Ok(bytes) => bytes,
                Err(err) => {
                    error!("Failed to load WASM module from {}: {}", path, err);
                    return Err(err);
                }
            };

            let instance = match create_wasm_instance(&wasm_bytes) {
                Ok(inst) => inst,
                Err(err) => {
                    error!("Failed to create WASM instance from {}: {}", path, err);
                    return Err(err);
                }
            };

            let result = execute_wasm_function(&instance, func_name).await?;
            info!("Execution result from {}: {}", path, result);

            Ok(())
        })
    }).collect();

    let results = join_all(tasks).await;
    for result in results {
        result??; // Unwrap result
    }

    Ok(())
}

/// Handles HTTP requests for executing WASM code.
///
/// # Arguments
///
/// * `req` - The incoming HTTP request.
///
/// # Returns
///
/// * `Result<Response<Body>, hyper::Error>` - Returns the HTTP response or an error.
async fn handle_request(req: Request<Body>) -> Result<Response<Body>, hyper::Error> {
    if req.method() == hyper::Method::POST {
        let body_bytes = hyper::body::to_bytes(req.into_body()).await.unwrap();
        let body_str = String::from_utf8(body_bytes.to_vec()).unwrap();
        let params: Vec<&str> = body_str.split(',').collect();
        
        if params.len() != 2 {
            return Ok(Response::new(Body::from("Invalid parameters")));
        }

        let wasm_path = params[0];
        let func_name = params[1];
        
        // Run the WASM module and execute the function
        match run_parallel_wasm_modules(vec![wasm_path], func_name).await {
            Ok(_) => Ok(Response::new(Body::from("Execution completed successfully"))),
            Err(e) => Ok(Response::new(Body::from(format!("Execution failed: {}", e)))),
        }
    } else {
        Ok(Response::new(Body::from("Invalid request method")))
    }
}

/// Main function to start the HTTP server.
///
/// # Returns
///
/// * `Result<(), Box<dyn Error>>` - Returns `Ok(())` on success or an error.
#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();
    
    // Load configuration from environment variables
    let addr = env::var("SERVER_ADDR").unwrap_or_else(|_| "127.0.0.1:3000".to_string());
    let addr: std::net::SocketAddr = addr.parse()?;

    // Define the HTTP server service
    let make_svc = make_service_fn(|_conn| async { Ok::<_, hyper::Error>(service_fn(handle_request)) });
    let server = Server::bind(&addr).serve(make_svc);

    info!("Listening on http://{}", addr);
    server.await?;
    
    Ok(())
}