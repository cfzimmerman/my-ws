use my_ws::ws::{
    event::{Event, EventAction},
    socket::To,
    ws_error::WsError,
    ws_io::{Io, Message},
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
        if let Err(error) = socket.send(Message::Text(message), To::All) {
            eprintln!("event failed: {:?}", error)
        };
    });

    let event_list: Vec<Event> = vec![Event::new("echo", echo)];

    let io = Io::build(&address, event_list).await?;
    io.listen().await;

    Ok(())
}
