use std::{collections::HashMap, sync::Arc};

use crate::ws::event::WsIoMsg;

use super::{
    event::{self, Event},
    ws_error::WsError,
};

use futures_channel::mpsc::{unbounded, UnboundedReceiver, UnboundedSender};
use futures_util::{future, pin_mut, StreamExt, TryStreamExt};
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};

#[derive(Debug)]
pub struct Client {
    pub messenger: Arc<Messenger>,
}

#[derive(Debug)]
/// The entity through which messages may be properly sent to the server.
pub struct Messenger {
    pub sender: UnboundedSender<Message>,
}

impl Messenger {
    pub fn send(&self, path: String, payload: serde_json::Value) -> Result<(), WsError> {
        let msg = serde_json::to_string(&event::WsIoMsg { path, payload })?;
        self.sender
            .unbounded_send(Message::Text(msg))
            .map_err(|e| WsError::SendError(format!("{}", e)))
    }
}

pub type ClientEventCxt = Arc<Messenger>;

impl Client {
    /// returns a new Client and a receiver instance. The receiver instance
    /// should be provided to 'listen' in order to properly emit WS messages.
    pub fn new() -> (Self, UnboundedReceiver<Message>) {
        let (tx, rx) = unbounded();
        let mg: ClientEventCxt = Arc::new(Messenger { sender: tx });
        (
            Client {
                messenger: mg.clone(),
            },
            rx,
        )
    }

    /// Attempts to establish a TCP connection with the provided url. If successful,
    /// sets up communication channels and begin listening for the provided events.
    pub async fn listen(
        &self,
        rx: UnboundedReceiver<Message>,
        server_url: &str,
        event_list: Vec<Event>,
    ) -> Result<(), WsError> {
        let (ws_stream, _) = connect_async(server_url).await?;
        let messenger = self.messenger.clone();
        tokio::spawn(async move {
            let (write, read) = ws_stream.split();
            let writer = rx.map(Ok).forward(write);

            let mut event_map = HashMap::new();
            for ev in event_list {
                event_map.insert(ev.path, ev.action);
            }
            let events = Arc::new(event_map);

            let catch_inbound = read.try_for_each(|msg| {
                let body = match msg {
                    Message::Text(txt) => txt,
                    _ => {
                        eprintln!("received unsupported message type: {:?}", &msg);
                        return future::ok(());
                    }
                };
                let ws_msg = match serde_json::from_str::<WsIoMsg>(&body) {
                    Ok(m) => m,
                    Err(e) => {
                        eprintln!("unable to parse message: {:?}", e);
                        return future::ok(());
                    }
                };
                let path: &str = &ws_msg.path;
                let action = match (*events).get(path) {
                    Some(closure) => closure,
                    None => {
                        eprintln!("received unrecognized path: {:?}", &ws_msg);
                        return future::ok(());
                    }
                };
                (*action)(event::Context::Client(messenger.clone()), ws_msg.payload);
                future::ok(())
            });

            pin_mut!(catch_inbound, writer);
            future::select(catch_inbound, writer).await;
        });
        Ok(())
    }
}
