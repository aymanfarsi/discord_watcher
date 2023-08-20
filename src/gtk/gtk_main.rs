#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use dotenv::dotenv;
use notify_rust::{Notification, Timeout};
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::model::user::OnlineStatus;
use serenity::model::voice::VoiceState;
use serenity::prelude::{Context, EventHandler, GatewayIntents};
use serenity::{async_trait, Client};
use std::env;
use tokio::sync::mpsc::{self, Sender};

pub mod app;
pub mod enums;
pub mod models;

use app::start_gtk_app;
use enums::ChannelMessage;

struct DiscordEventHandler {
    tx: Sender<ChannelMessage>,
}

#[async_trait]
impl EventHandler for DiscordEventHandler {
    async fn message(&self, ctx: Context, msg: Message) {
        if msg.content == "!ping" {
            println!("Shard {}", ctx.shard_id);

            let res = msg.channel_id.say(&ctx.http, "Pong!").await;
            match res {
                Ok(_) => println!("Message sent"),
                Err(why) => println!("Error sending message: {:?}", why),
            }
        }
    }

    async fn ready(&self, ctx: Context, ready: Ready) {
        ctx.set_presence(None, OnlineStatus::Invisible).await;
        println!("{} is connected!", ready.user.name);
    }

    async fn voice_state_update(&self, ctx: Context, _: Option<VoiceState>, new: VoiceState) {
        let new_user = match new.user_id.to_user_cached(&ctx.cache).await {
            Some(user) => user,
            None => new.user_id.to_user(&ctx.http).await.unwrap(),
        };

        let new_voice_channel = match new.channel_id {
            Some(channel_id) => match channel_id.to_channel_cached(&ctx.cache) {
                Some(channel) => channel.guild().unwrap(),
                None => new
                    .channel_id
                    .unwrap()
                    .to_channel(&ctx.http)
                    .await
                    .unwrap()
                    .guild()
                    .unwrap(),
            },
            None => {
                Notification::new()
                    .summary("Discord Watcher")
                    .body(&format!("{} left a channel", new_user.name))
                    .timeout(Timeout::Milliseconds(500))
                    .auto_icon()
                    .sound_name("Default")
                    .show()
                    .unwrap();

                self.tx
                    .send(ChannelMessage::UserLeftChannel(new_user.name.clone()))
                    .await
                    .unwrap();

                return;
            }
        };

        Notification::new()
            .summary("Discord Watcher")
            .body(&format!(
                "{} joined {}",
                new_user.name, new_voice_channel.name
            ))
            .timeout(Timeout::Milliseconds(500))
            .auto_icon()
            .sound_name("Default")
            .show()
            .unwrap();

        self.tx
            .send(ChannelMessage::UserJoinedChannel(
                new_user.name.clone(),
                new_voice_channel.name.clone(),
            ))
            .await
            .unwrap();
    }
}

#[tokio::main]
async fn main() {
    // * Load discord token from .env
    dotenv().ok();
    let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");

    // * Create GatewayIntents
    let intents = GatewayIntents::GUILD_VOICE_STATES
        | GatewayIntents::GUILD_PRESENCES
        | GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT;

    // * Create channel
    let (tx, rx) = mpsc::channel::<ChannelMessage>(100);

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

    // * Create gtk4 app
    start_gtk_app(rx);
}
