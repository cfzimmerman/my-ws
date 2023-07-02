use crate::ws::{event::Message, server_io::ClientMap, ws_error::WsError};
use futures_channel::mpsc::TrySendError;
use std::net::SocketAddr;

/// To - Who to send a WS message to.
/// Origin: Send to the client we just received the message from.
/// NonOrigin: Send to everyone except the client we just received the message from.
/// All: Send to everyone connected to the server.
#[derive(Debug)]
pub enum To {
    Origin,
    NonOrigin,
    All,
}

/// Socket: an object used to handle a client's connection with the server.
#[derive(Debug)]
pub struct Socket {
    address: SocketAddr,
    clients: ClientMap,
}

impl Socket {
    pub fn new(clients: &ClientMap, address: SocketAddr) -> Socket {
        Socket {
            address,
            clients: clients.clone(),
        }
    }

    /// send: sends a message to everyone connected to the server matching the
    /// specified "To" scope.
    /// Prints to stderr if a message fails to send. May also return WsError.
    /// If messages failed to send, returns a vector of clients who didn't receive
    /// messages.
    pub async fn send(&self, msg: Message, to: To) -> Result<(), WsError> {
        let clients = self.clients.lock().await;
        let mut failed: Vec<String> = vec![];
        let mut send_failed = |address: String, error: TrySendError<Message>| {
            eprintln!("send error: {error}");
            failed.push(address)
        };
        match to {
            To::Origin => {
                let ws_out = clients
                    .get(&self.address)
                    .ok_or_else(|| WsError::ClientNotFound)?;
                if let Err(err) = ws_out.unbounded_send(msg) {
                    send_failed(self.address.to_string(), err)
                }
            }
            To::NonOrigin => clients
                .iter()
                .filter(|(&client_addr, _)| client_addr != self.address)
                .for_each(|(addr, ws_out)| {
                    if let Err(err) = ws_out.unbounded_send(msg.clone()) {
                        send_failed(addr.to_string(), err)
                    }
                }),
            To::All => clients.iter().for_each(|(addr, ws_out)| {
                if let Err(err) = ws_out.unbounded_send(msg.clone()) {
                    send_failed(addr.to_string(), err)
                }
            }),
        };
        if failed.len() > 0 {
            return Err(WsError::FailedToSend(failed));
        }
        Ok(())
    }
}
