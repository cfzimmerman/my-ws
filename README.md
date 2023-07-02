# My-WS

[Demo](https://youtube.com/shorts/MNtP16yU42w?feature=share)

This is a multithreaded, event-oriented WebSocket library built on top
of Tokio and Tungstenite. Both server and client modules are provided.

Since this was one of my first real projects with multithreaded async programming
in Rust, the library isn't well-optimized for performance. It's certainly usable, though.

Calling `cargo run` on the crate will launch the example in `main.rs`, which starts a server
instance, a client instance, and runs an echo event between them. For more details on usage,
checkout `ws::examples`!
