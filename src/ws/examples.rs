use serde_json::{json, Value};

use crate::ws::{
    client_io::Client,
    event::{Context, Event, EventAction, Message, WsIoMsg},
    server_io::Io,
    socket::To,
    ws_error::WsError,
};

pub async fn server_ex(addr: &str) -> Result<(), WsError> {
    // A boxed async function that can be associated with a path.
    // When the path is triggered, the function extracts the message and
    // sends it back to every connected client.
    // Because every path shares the same declaration of events, it's wrapped
    // in an Arc Mutex.
    let server_echo: EventAction = Box::new(|socket, message| {
        tokio::spawn(async move {
            let socket = match socket {
                Context::Server(sk) => sk,
                Context::Client(_) => return,
            };
            let socket = socket.lock().await;
            if let Value::String(msg) = message {
                let copied_msg = msg.clone();
                println!("server received: {}", msg);
                let response = WsIoMsg {
                    path: "echo".to_string(),
                    payload: Value::String(msg),
                };
                let response = match serde_json::to_string(&response) {
                    Ok(res) => res,
                    Err(e) => {
                        eprint!("failed to serialize response: {:?}", e);
                        return;
                    }
                };
                match socket.send(Message::Text(response), To::All).await {
                    Ok(_) => println!("server sent: {}", copied_msg),
                    Err(e) => eprintln!("event failed: {:?}", e),
                };
            }
        });
    });
    let server_event_list: Vec<Event> = vec![Event::new("echo", server_echo)];
    // The server will establish a listener and respond to path events with
    // readable payloads. Unknown paths or unreadable payloads are currently
    // ignored.
    let io = match Io::build(addr, server_event_list).await {
        Ok(ws_server) => ws_server,
        Err(e) => {
            eprintln!("failed to mount server: {:?}", e);
            return Err(e);
        }
    };
    io.listen().await;
    Ok(())
}

pub async fn client_ex(addr: &str, msg: &str) -> Result<(), WsError> {
    // Path-Event boxed closures are practically identical between
    // the client and server. Client closures
    // receive a messenger, which provides the ability to send
    // properly-formatted messages to the server.
    // A single messenger is shared by all client handlers and is thus
    // wrapped in an Arc.
    let client_echo: EventAction = Box::new(|messenger, message| {
        tokio::spawn(async move {
            let messenger = match messenger {
                Context::Server(_) => return,
                Context::Client(mg) => mg,
            };
            if let serde_json::value::Value::String(msg) = message {
                println!("client received: {}", msg);
                let unlistened_path = "echo back".to_string();
                let payload = json!(msg);
                if let Err(error) = (*messenger).send(unlistened_path, payload) {
                    eprintln!("event failed: {:?}", error)
                };
            };
        });
    });

    let client_event_list: Vec<Event> = vec![Event::new("echo", client_echo)];

    // The client's transmitter sends to a receiver. When that receiver is passed
    // to `listen`, events transmitted from the client are forwarded as outbound
    // WS messages.
    let (client, rx) = Client::new();
    client.listen(rx, addr, client_event_list).await?;
    let messenger = client.messenger.clone();

    // Here, we send a single message to the server along the "echo" path

    match messenger.send("echo".to_string(), json!(msg)) {
        Ok(_) => println!("client sent: {}", msg),
        Err(e) => eprintln!("failed to send message: {:?}", e),
    };
    Ok(())
}
