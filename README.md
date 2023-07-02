# My-WS

This is my own implementation of a high-level WebSocket client built on top
of Tokio and Tungstenite. Design is heavily inspired by JS Socket.io.

Since this was one of my first real encounters with multithreaded async programming
in Rust, the library isn't well-optimized for performance. It's certainly usable, though.

Calling `cargo run` on the crate will launch the example in `main.rs`, which starts a server
instance, a client instance, and runs an echo event between them. For more details on usage,
checkout `ws::examples`!
