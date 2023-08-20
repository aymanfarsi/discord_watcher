#![warn(clippy::all, rust_2018_idioms)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

pub mod egui {
    pub mod egui_app;
    pub mod top_bar;
}
pub mod discord;
pub mod enums;
pub mod models;
pub mod utils;

#[cfg(not(target_arch = "wasm32"))]
fn main() -> eframe::Result<()> {
    use std::env;

    use dotenv::dotenv;
    use eframe::{HardwareAcceleration, Theme};
    use serenity::{prelude::GatewayIntents, Client};
    use tokio::{runtime::Runtime, sync::mpsc};

    use crate::{discord::DiscordEventHandler, egui::egui_app::AppModel, enums::ChannelMessage};

    // * Create tokio runtime
    let rt = Runtime::new().expect("Unable to create Runtime");
    let _enter = rt.enter();

    // * Load discord token from .env
    dotenv().ok();
    let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");

    // * Create GatewayIntents
    let intents = GatewayIntents::GUILD_VOICE_STATES | GatewayIntents::GUILD_PRESENCES;
    // | GatewayIntents::GUILD_MESSAGES
    // | GatewayIntents::DIRECT_MESSAGES
    // | GatewayIntents::MESSAGE_CONTENT;

    // * Create channel
    let (tx, rx) = mpsc::channel::<ChannelMessage>(1);

    // * Initiate event handler struct
    let event_handler = DiscordEventHandler { tx };

    // * Create Discord thread
    tokio::spawn(async move {
        let mut client = Client::builder(&token, intents)
            .event_handler(event_handler)
            .await
            .expect("Err creating client");

        if let Err(why) = client.start_shards(1).await {
            println!("Client error: {:?}", why);
        }
    });

    // * Initialize native options
    let native_options = eframe::NativeOptions {
        initial_window_size: Some(eframe::egui::Vec2::new(360.0, 360.0)),
        min_window_size: Some(eframe::egui::Vec2::new(100.0, 100.0)),
        always_on_top: false,
        maximized: false,
        centered: true,
        transparent: true,
        default_theme: Theme::Dark,
        follow_system_theme: false,
        hardware_acceleration: HardwareAcceleration::Preferred,
        // icon_data: Some(
        //     IconData::try_from_png_bytes(&include_bytes!("../assets/icon-256.png")[..]).unwrap(),
        // ),
        ..Default::default()
    };

    // * Run egui app
    eframe::run_native(
        "Discord Watcher",
        native_options,
        Box::new(|cc| Box::new(AppModel::new(cc, rx))),
    )
}
