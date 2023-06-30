use my_ws::ws::{
    client_io::Client,
    event::{Context, Event, EventAction},
    server_io::{Io, Message},
    socket::To,
    ws_error::WsError,
};
use serde_json::json;
use std::{boxed::Box, env, time};

/// Example
///
/// Events are specified as boxed functions mapped to a string path name.
/// In this example, a single path is mounted on the WS listener.
/// While the server is listening, all incoming messages of path `echo` are echoed
/// back to all connected clients.
#[tokio::main]
async fn main() -> Result<(), WsError> {
    let server_address = env::args()
        .nth(1)
        .unwrap_or_else(|| "127.0.0.1:5445".to_string());
    let client_send_to = server_address.clone();

    let server_echo: EventAction = Box::new(|socket, message| {
        tokio::spawn(async move {
            let socket = match socket {
                Context::Server(sk) => sk,
                Context::Client(_) => return,
            };
            let socket = socket.lock().await;
            if let serde_json::value::Value::String(msg) = message {
                println!("server received: {:?}", msg);
                if let Err(error) = socket.send(Message::Text(msg), To::All).await {
                    eprintln!("event failed: {:?}", error)
                };
            }
        });
    });
    let server_event_list: Vec<Event> = vec![Event::new("echo", server_echo)];

    tokio::spawn(async move {
        let io = match Io::build(&server_address, server_event_list).await {
            Ok(ws_server) => ws_server,
            Err(e) => {
                eprintln!("failed to mount server: {:?}", e);
                return;
            }
        };
        io.listen().await;
    });

    let client_echo: EventAction = Box::new(|messenger, message| {
        tokio::spawn(async move {
            let messenger = match messenger {
                Context::Server(_) => return,
                Context::Client(mg) => mg,
            };
            if let serde_json::value::Value::String(msg) = message {
                println!("client received: {:?}", msg);
                let unlistened_path = "echo back".to_string();
                let payload = json!(msg);
                if let Err(error) = (*messenger).send(unlistened_path, payload) {
                    eprintln!("event failed: {:?}", error)
                };
            };
        });
    });

    let client_event_list: Vec<Event> = vec![Event::new("echo", client_echo)];
    let (client, rx) = Client::new();

    // Wait for the backend to be live before connecting the client
    std::thread::sleep(time::Duration::from_millis(2500));

    todo!("push client listening into its own thread so it doesn't need to be awaited");
    client.listen(rx, &client_send_to, client_event_list);

    let messenger = client.messenger.clone();
    tokio::spawn(async move {
        // Wait for the client to be live before sending a message
        std::thread::sleep(time::Duration::from_millis(2500));
        if let Err(e) = messenger.send("echo".to_string(), json!("echo out!")) {
            eprintln!("failed to send message: {:?}", e)
        };
    });
    Ok(())
}
