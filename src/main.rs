use my_ws::ws::{examples, ws_error::WsError};
use std::{env, time};

/// Example
///
/// Starts the server example, waits for the server to be ready, and
/// starts the client example.
///
/// The client example sends a message to the server along path `echo`.
/// The server receives the message on the `echo` path and echoes it
/// back to all clients (we just have one). The client hears the echo
/// and returns a message to the server on an unlistened path, which
/// prevents looping.
///
/// After running `cargo run`, echoes should be printed to stdout.
/// The client waits a few seconds before initializing the first echo.
#[tokio::main]
async fn main() -> Result<(), WsError> {
    let server_address = env::args()
        .nth(1)
        .unwrap_or_else(|| "127.0.0.1:5445".to_string());
    let client_address = format!("ws://{}/", server_address);

    tokio::spawn(async move {
        if let Err(e) = examples::server_ex(&server_address).await {
            eprintln!("failed to start the server example: {:?}", e);
        };
    });

    // Wait for the backend to be live before attempting to connect the client
    std::thread::sleep(time::Duration::from_millis(500));

    let message = "echo out!";
    examples::client_ex(&client_address, message).await?;

    // Wait for the WS conversation to take place
    std::thread::sleep(time::Duration::from_millis(500));

    println!("\nDone listening");
    Ok(())
}
