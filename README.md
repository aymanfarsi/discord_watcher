# Discord Watcher

---

## Project description

`Discord Watcher` is a Rust application that uses [egui](https://github.com/emilk/egui) for the UI and [serenity](https://github.com/serenity-rs/serenity) to communicate with the Discord API.

## Features

- Minimal application (low resources used)
- Builds to a single executable
- Listens to voice chat changes and logs the event in the app (WIP)
- Uses Discord bot to listen to events (bot is invisible in the server)
- Plays a small notification sound when an event occurs
- Can specify Discord bot token using `.env` file

## Dependencies

The only dependencies are rust and cargo. For the crates, it needs the following:

- `serenity`: 0.11.6 (features: client, gateway, rustls_backend, model, cache)
- `tokio`: 1.29.1 (features: macros, ru-multi-thread)
- `dotenv`: 0.15.0
- `egui` & `eframe`: 0.22.0
- `egui-phosphor`: 0.2.0
- `rodio`: 0.17.1

## Build & Run

1. Clone the repo
2. Navigate to the project root
3. run `cargo build -r` (release mode)
4. You will find your executable in diractory `$crate/target/release/`


