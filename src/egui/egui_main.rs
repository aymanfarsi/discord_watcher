pub fn start_egui() -> eframe::Result<()> {
    use std::env;

    use ::egui::{vec2, ViewportBuilder};
    use dotenv::dotenv;
    use eframe::{icon_data::from_png_bytes, HardwareAcceleration};
    use serenity::{prelude::GatewayIntents, Client};
    use tokio::{runtime::Runtime, sync::mpsc};

    use crate::{discord::DiscordEventHandler, egui::app::AppModel, enums::ChannelMessage};

    // * Create tokio runtime
    let rt = Runtime::new().expect("Unable to create Runtime");
    let _enter = rt.enter();

    // * Load discord token from .env
    dotenv().ok();
    let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");

    // * Create channel
    let (tx, rx) = mpsc::channel::<ChannelMessage>(1);

    // * Initialize native options
    let native_options = eframe::NativeOptions {
        viewport: ViewportBuilder::default()
            .with_inner_size(vec2(360.0, 360.0))
            .with_min_inner_size(vec2(100.0, 100.0))
            .with_transparent(true)
            .with_transparent(true)
            .with_icon(
                from_png_bytes(&include_bytes!("../../assets/discord_watcher.png")[..])
                    .expect("Failed to load icon"),
            ),
        centered: true,
        hardware_acceleration: HardwareAcceleration::Preferred,

        ..Default::default()
    };

    // * Run egui app
    eframe::run_native(
        "Discord Watcher",
        native_options,
        Box::new(|cc| {
            // * Create GatewayIntents
            let intents = GatewayIntents::GUILD_VOICE_STATES | GatewayIntents::GUILD_PRESENCES;
            // | GatewayIntents::GUILD_MESSAGES
            // | GatewayIntents::DIRECT_MESSAGES
            // | GatewayIntents::MESSAGE_CONTENT;

            // * Initiate event handler struct
            let event_handler = DiscordEventHandler {
                tx,
                ctx: cc.egui_ctx.clone(),
            };

            // * Create Discord thread
            tokio::spawn(async move {
                let mut client = Client::builder(&token, intents)
                    .event_handler(event_handler)
                    .await
                    .expect("Err creating client");

                if let Err(why) = client.start_shards(1).await {
                    eprintln!("Client error: {:?}", why);
                }
            });

            Ok(Box::new(AppModel::new(cc, rx)))
        }),
    )
}
