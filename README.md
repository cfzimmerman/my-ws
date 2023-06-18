# My-WS

To build my Node-Rust WS benchmark project, I needed a WebSocket client.
While I could have used a package, I wanted to try writing my own.

This crate is very-much inspired by the design of JS Socket.io. A server mounts
listeners on paths and passes listeners the context they need to respond.
I wrote this pretty quickly, so it's missing some essential Socket.io features
like fallback HTTP polling and client room designations, but I'm overall quite
happy with the project. Because of modular design, extending to include those
and other features should be fairly straightforward if needed.

If you'd like to try it, clone the repo, `cargo run`, and use a WS client to start
passing messages around. Note, messages in the main.rs example will be rejected unless
they're valid JSON that can be parsed into the `ws::ws_io::WsIoMsg` struct specification:

```
{
  path: "echo",
  payload: "Hello my-ws!"
}
```
