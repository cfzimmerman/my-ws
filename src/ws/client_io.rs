use std::sync::Arc;

use super::{event, ws_error::WsError};

use futures_channel::mpsc::{unbounded, UnboundedSender};
use futures_util::{future, pin_mut, StreamExt, TryStreamExt};
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};

#[derive(Debug)]
pub struct Client {}

#[derive(Debug)]
pub struct Messenger {
    sender: UnboundedSender<Message>,
}

/*

To do:
Accept and properly handle closures for event handlers.
Add a client example.

*/

impl Messenger {
    fn send(&self, path: String, payload: serde_json::Value) -> Result<(), WsError> {
        let msg = serde_json::to_string(&event::WsIoMsg { path, payload })?;
        self.sender
            .unbounded_send(Message::Text(msg))
            .map_err(|e| WsError::SendError(format!("{}", e)))
    }
}

pub type ClientEventCxt = Arc<Messenger>;

impl Client {
    async fn build(server_url: &str) -> Result<Self, WsError> {
        let (ws_stream, _) = connect_async(server_url).await?;
        let (tx, rx) = unbounded();
        let (write, read) = ws_stream.split();
        let writer = rx.map(Ok).forward(write);

        let mg: ClientEventCxt = Arc::new(Messenger { sender: tx });

        let catch_inbound = read.try_for_each(|msg| {
            println!("{:?}", msg);
            future::ok(())
        });

        pin_mut!(catch_inbound, writer);
        future::select(catch_inbound, writer).await;

        Ok(Client {})
    }
}
