use super::{ws_error::WsError, ws_io};

use futures_channel::mpsc::{unbounded, UnboundedSender};
use futures_util::{future, pin_mut, StreamExt, TryStreamExt};
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};

struct Client {
    sender: UnboundedSender<Message>,
}

impl Client {
    async fn build(server_url: &str) -> Result<Self, WsError> {
        let (ws_stream, _) = connect_async(server_url).await?;
        let (tx, rx) = unbounded();
        let (write, read) = ws_stream.split();
        let writer = rx.map(Ok).forward(write);

        let catch_inbound = read.try_for_each(|msg| {
            println!("{:?}", msg);
            future::ok(())
        });

        pin_mut!(catch_inbound, writer);
        future::select(catch_inbound, writer).await;

        Ok(Client { sender: tx })
    }

    fn send(&self, path: String, payload: serde_json::Value) -> Result<(), WsError> {
        let msg = serde_json::to_string(&ws_io::WsIoMsg { path, payload })?;
        self.sender
            .unbounded_send(Message::Text(msg))
            .map_err(|e| WsError::SendError(format!("{}", e)))
    }
}
