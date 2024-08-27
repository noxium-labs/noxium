use async_std::task;
use serde::{Deserialize, Serialize};
use socketio::{SocketIo, Event};
use tide::{Request, Response, Server};
use tide::utils::After;

#[derive(Debug, Serialize, Deserialize)]
struct CustomMessage {
    user: String,
    content: String,
}

async fn handle_request(req: Request<()>) -> tide::Result {
    let mut res = Response::new(200);
    res.set_body("Socket.IO Server is running!");
    Ok(res)
}

fn setup_socketio_events(socketio: &mut SocketIo) {
    socketio.on("connect", |data| {
        println!("Client connected: {:?}", data);
    });

    socketio.on("disconnect", |data| {
        println!("Client disconnected: {:?}", data);
    });

    socketio.on("message", |data| {
        println!("Received message: {:?}", data);
        // Broadcast received message to all clients
        socketio.broadcast("broadcast", data.clone()).unwrap();
    });

    socketio.on("custom_event", |data: String| {
        println!("Received custom event: {}", data);
    });

    socketio.on("send_custom_message", |message: CustomMessage| {
        println!("Received custom message: {:?}", message);
        // Example response back to the client
        let response_message = format!("Hello, {}! You sent: {}", message.user, message.content);
        socketio.emit("custom_response", response_message).unwrap();
    });
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut socketio = SocketIo::new();
    setup_socketio_events(&mut socketio);

    let mut app = tide::new();
    app.at("/").get(handle_request);

    app.with(After(|res: Response| {
        println!("Response sent: {:?}", res);
        async { Ok(res) }
    }));

    let addr = "127.0.0.1:8080";
    let listener = tokio::net::TcpListener::bind(addr).await?;
    println!("Server running on {}", addr);

    let server = Server::new(socketio);
    task::block_on(server.listen(listener))?;

    Ok(())
}