use serenity::{
    async_trait,
    model::{
        prelude::{ChannelType, Ready},
        user::OnlineStatus,
        voice::VoiceState,
    },
    prelude::{Context, EventHandler},
};
use tokio::sync::mpsc::Sender;

use crate::{enums::ChannelMessage, utils::{play_sound, push_notification}};

pub struct DiscordEventHandler {
    pub tx: Sender<ChannelMessage>,
    pub ctx: egui::Context,
}

#[async_trait]
impl EventHandler for DiscordEventHandler {
    async fn ready(&self, ctx: Context, ready: Ready) {
        ctx.set_presence(None, OnlineStatus::Invisible).await;
        self.tx
            .send(ChannelMessage::BotConnected(Box::new(ready.clone())))
            .await
            .unwrap();
        self.ctx.request_repaint();

        let guild = ready.guilds[0]
            .id
            .to_partial_guild(&ctx.http)
            .await
            .unwrap();

        let channels = guild.channels(&ctx.http).await.unwrap();
        for (_, channel) in channels {
            if channel.kind == ChannelType::Voice {
                let joined_members = match channel.members(&ctx.cache).await {
                    Ok(members) => members,
                    Err(_) => {
                        let guild = match channel.guild(&ctx.cache) {
                            Some(guild) => guild,
                            None => continue,
                        };
                        guild
                            .voice_states
                            .values()
                            .filter_map(|v| {
                                v.channel_id.and_then(|c| {
                                    if c == channel.id {
                                        guild.members.get(&v.user_id).cloned()
                                    } else {
                                        None
                                    }
                                })
                            })
                            .collect()
                    }
                };

                for member in joined_members {
                    push_notification(&format!(
                        "{} is already in {}",
                        member.user.name.clone(),
                        channel.name.clone()
                    ));
                    play_sound();

                    self.tx
                        .send(ChannelMessage::UserAlreadyInChannel(
                            member.user.name.clone(),
                            channel.name.clone(),
                        ))
                        .await
                        .unwrap();
                    self.ctx.request_repaint();
                }
            }
        }
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
                push_notification(&format!("{} left a channel", new_user.name.clone()));
                play_sound();

                self.tx
                    .send(ChannelMessage::UserLeftChannel(new_user.name.clone()))
                    .await
                    .unwrap();
                self.ctx.request_repaint();

                return;
            }
        };

        push_notification(&format!(
            "{} joined {}",
            new_user.name.clone(),
            new_voice_channel.name.clone()
        ));
        play_sound();

        self.tx
            .send(ChannelMessage::UserJoinedChannel(
                new_user.name.clone(),
                new_voice_channel.name.clone(),
            ))
            .await
            .unwrap();
        self.ctx.request_repaint();
    }
}
