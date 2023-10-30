# 🎮 quic_game 🎮

Welcome to `quic_game`, a Rust-based game project that utilizes the QUIC protocol for networking!

This project is a personal endeavor for learning and practicing Test-Driven Development (TDD).

## 📦 Dependencies 📦

- **bevy**: `0.11.3`
- **bevy_quinnet**: `0.5.0`
- **serde**: `1.0.190`

## 🚀 Getting Started 🚀

### Client

The client application can be found [here](https://github.com/donedgardo/quic_game/blob/main/src/bin/client.rs). It uses the `bevy` game engine and the `ClientPlugin` to establish a connection to the server.

### Server

The server application is located [here](https://github.com/donedgardo/quic_game/blob/main/src/bin/server.rs). It also uses the `bevy` game engine and the `ServerPlugin` to listen for incoming client connections.

### Shared

Shared resources and messages between the client and server can be found [here](https://github.com/donedgardo/quic_game/blob/main/src/shared/mod.rs).

## 🤝 Contributing 🤝

Feel free to fork the repository, make changes, and submit pull requests. Let's make this game even more awesome!
