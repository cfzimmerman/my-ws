use my_ws::ws::{
    event::{Context, Event, EventAction},
    server_io::{Io, Message},
    socket::To,
    ws_error::WsError,
};
use std::{boxed::Box, env};

/// Example
///
/// Events are specified as boxed functions mapped to a string path name.
/// In this example, a single path is mounted on the WS listener.
/// While the server is listening, all incoming messages of path `echo` are echoed
/// back to all connected clients.
#[tokio::main]
async fn main() -> Result<(), WsError> {
    let address = env::args()
        .nth(1)
        .unwrap_or_else(|| "127.0.0.1:5445".to_string());

    let echo: EventAction = Box::new(|socket, message| {
        tokio::spawn(async move {
            let socket = match socket {
                Context::Server(sk) => sk,
                Context::Client(_) => return,
            };
            let socket = socket.lock().await;
            if let serde_json::value::Value::String(msg) = message {
                if let Err(error) = socket.send(Message::Text(msg), To::All).await {
                    eprintln!("event failed: {:?}", error)
                };
            }
        });
        ()
    });

    let event_list: Vec<Event> = vec![Event::new("echo", echo)];

    let io = Io::build(&address, event_list).await?;
    io.listen().await;

    Ok(())
}
