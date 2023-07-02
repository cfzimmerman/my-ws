use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{collections::HashMap, sync::Arc};
pub use tokio_tungstenite::tungstenite::protocol::Message;

use super::{client_io::ClientEventCxt, server_io::ServerEventCxt};

/// WsIoMsg: The fundamental unit passed between client and server. Any
/// received package not obeying this struct will be ignored.
#[derive(Debug, Serialize, Deserialize)]
pub struct WsIoMsg {
    pub path: String,
    pub payload: Value,
}

#[derive(Debug)]
pub enum Context {
    Server(ServerEventCxt),
    Client(ClientEventCxt),
}

/// Event: an object specifying to listen for a certain type of
/// event with instructions for how to handle the event.
pub struct Event {
    pub path: &'static str,
    pub action: EventAction,
}

/// EventMap: A thread-shared hash mapping each path to its associated action.
pub type EventMap = Arc<HashMap<&'static str, EventAction>>;

/// EventAction: Given a Socket instance and the string received by the server,
/// performs some desired action.
pub type EventAction = Box<dyn Fn(Context, Value) -> () + Send + Sync>;

impl Event {
    /// new: Creates a new Event object.
    pub fn new(path: &'static str, action: EventAction) -> Event {
        Event { path, action }
    }
}
