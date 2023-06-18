use crate::ws::{
    event::{Event, EventMap},
    socket::Socket,
    ws_error::WsError,
};
use futures_channel::mpsc::{unbounded, UnboundedSender};
use futures_util::{future, pin_mut, stream::TryStreamExt, StreamExt};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{collections::HashMap, net::SocketAddr, sync::Arc};
use tokio::{
    net::{TcpListener, TcpStream},
    sync::Mutex,
};

pub use tokio_tungstenite::tungstenite::protocol::Message;

/// WsIoMsg: The fundamental unit passed between client and server. Any
/// received package not obeying this struct will be ignored.
#[derive(Serialize, Deserialize)]
pub struct WsIoMsg {
    path: String,
    payload: Value,
}

/// Io: the main socket server instance.
pub struct Io {
    listener: TcpListener,
    clients: ClientMap,
    events: EventMap,
}

type Tx = UnboundedSender<Message>;

/// ClientMap: a thread-shareable hash mapping client addresses with
/// their send-message connection.
pub type ClientMap = Arc<Mutex<HashMap<SocketAddr, Tx>>>;

impl Io {
    /// build: attempt to bind a TCP listener at the given address and set up the WS client.
    pub async fn build(address: &str, event_list: Vec<Event>) -> Result<Io, WsError> {
        let listener = TcpListener::bind(&address).await?;
        println!("Listening on {}", address);
        let mut event_map = HashMap::new();
        for ev in event_list {
            event_map.insert(ev.path, ev.action);
        }
        Ok(Io {
            listener,
            clients: ClientMap::new(Mutex::new(HashMap::new())),
            events: EventMap::new(std::sync::Mutex::new(event_map)),
        })
    }

    /// listen: begin accepting and handling client connection requests.
    pub async fn listen(&self) -> () {
        while let Ok((stream, addr)) = self.listener.accept().await {
            tokio::spawn(Io::manage_connection(
                self.clients.clone(),
                self.events.clone(),
                stream,
                addr,
            ));
        }
    }

    /// manage_connection: manage the server's relationship with connected clients,
    /// handling events as they're received.
    async fn manage_connection(
        client_map: ClientMap,
        event_map: EventMap,
        stream: TcpStream,
        addr: SocketAddr,
    ) -> Result<(), WsError> {
        let ws_stream = tokio_tungstenite::accept_async(stream).await?;
        let (sender, receiver) = unbounded();
        client_map.lock().await.insert(addr, sender);
        let (outbound, inbound) = ws_stream.split();

        /*
        The socket is a Mutex because multiple requests could come in from
        one user at the same time. If async processing is involved, this allows
        us to properly pass the socket client to handlers without lifetime issues.
        */
        let socket_shareable = Arc::new(Mutex::new(Socket::new(&client_map, addr)));

        let catch_inbound = inbound.try_for_each(|msg| {
            let socket = socket_shareable.clone();
            let str_msg = if let Message::Text(txt) = msg {
                txt
            } else {
                return future::ok(());
            };
            let from_client = if let Ok(pth) = serde_json::from_str::<WsIoMsg>(&str_msg) {
                pth
            } else {
                eprintln!("unable to parse message: {:?}", str_msg);
                return future::ok(());
            };
            let path: &str = &from_client.path;
            let events = if let Ok(ev) = event_map.lock() {
                ev
            } else {
                return future::ok(());
            };
            if let Some(handler) = events.get(path) {
                // All the conditions are correct. We can call the event.
                (*handler)(socket, from_client.payload);
            }
            future::ok(())
        });

        let receive_from_others = receiver.map(Ok).forward(outbound);
        pin_mut!(catch_inbound, receive_from_others);
        future::select(catch_inbound, receive_from_others).await;

        println!("{} disconnected", &addr);
        client_map.lock().await.remove(&addr);

        Ok(())
    }
}
