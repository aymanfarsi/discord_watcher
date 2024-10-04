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

use crate::{
    enums::ChannelMessage,
    utils::{play_sound, push_notification},
};

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

    async fn voice_state_update(
        &self,
        ctx: Context,
        old_state: Option<VoiceState>,
        new_state: VoiceState,
    ) {
        let old_user = match old_state.clone() {
            Some(old) => match old.user_id.to_user_cached(&ctx.cache).await {
                Some(user) => Some(user),
                None => Some(old.user_id.to_user(&ctx.http).await.unwrap()),
            },
            None => None,
        };
        let new_user = match new_state.user_id.to_user_cached(&ctx.cache).await {
            Some(user) => user,
            None => new_state.user_id.to_user(&ctx.http).await.unwrap(),
        };

        let old_voice_channel = match old_state.clone() {
            Some(old) => match old.channel_id {
                Some(channel_id) => match channel_id.to_channel_cached(&ctx.cache) {
                    Some(channel) => Some(channel.guild().unwrap()),
                    None => Some(
                        old.channel_id
                            .unwrap()
                            .to_channel(&ctx.http)
                            .await
                            .unwrap()
                            .guild()
                            .unwrap(),
                    ),
                },
                None => None,
            },
            None => None,
        };
        let new_voice_channel = match new_state.channel_id {
            Some(channel_id) => match channel_id.to_channel_cached(&ctx.cache) {
                Some(channel) => Some(channel.guild().unwrap()),
                None => Some(
                    new_state
                        .channel_id
                        .unwrap()
                        .to_channel(&ctx.http)
                        .await
                        .unwrap()
                        .guild()
                        .unwrap(),
                ),
            },
            None => None,
        };

        // check if user joined a voice channel, muted, deafened, moved to another voice channel, or left a voice channel
        if old_state.is_none() && new_state.channel_id.is_some() {
            push_notification(&format!(
                "{} joined {}",
                new_user.name.clone(),
                new_voice_channel.clone().unwrap().name.clone()
            ));
            play_sound();

            self.tx
                .send(ChannelMessage::UserJoinedChannel(
                    new_user.name.clone(),
                    new_voice_channel.clone().unwrap().name.clone(),
                ))
                .await
                .unwrap();
            self.ctx.request_repaint();
        } else if old_state.is_some() && new_state.channel_id.is_some() {
            if old_state.clone().unwrap().self_mute != new_state.self_mute {
                if new_state.self_mute {
                    push_notification(&format!(
                        "{} muted themselves in {}",
                        new_user.name.clone(),
                        new_voice_channel.clone().unwrap().name.clone()
                    ));
                    play_sound();

                    self.tx
                        .send(ChannelMessage::UserMuted(
                            new_user.name.clone(),
                            new_voice_channel.clone().unwrap().name.clone(),
                        ))
                        .await
                        .unwrap();
                    self.ctx.request_repaint();
                } else {
                    push_notification(&format!(
                        "{} unmuted themselves in {}",
                        new_user.name.clone(),
                        new_voice_channel.clone().unwrap().name.clone()
                    ));
                    play_sound();

                    self.tx
                        .send(ChannelMessage::UserUnmuted(
                            new_user.name.clone(),
                            new_voice_channel.clone().unwrap().name.clone(),
                        ))
                        .await
                        .unwrap();
                    self.ctx.request_repaint();
                }
            } else if old_state.clone().unwrap().self_deaf != new_state.self_deaf {
                if new_state.self_deaf {
                    push_notification(&format!(
                        "{} deafened themselves in {}",
                        new_user.name.clone(),
                        new_voice_channel.clone().unwrap().name.clone()
                    ));
                    play_sound();

                    self.tx
                        .send(ChannelMessage::UserDeafened(
                            new_user.name.clone(),
                            new_voice_channel.clone().unwrap().name.clone(),
                        ))
                        .await
                        .unwrap();
                    self.ctx.request_repaint();
                } else {
                    push_notification(&format!(
                        "{} undeafened themselves in {}",
                        new_user.name.clone(),
                        new_voice_channel.clone().unwrap().name.clone()
                    ));
                    play_sound();

                    self.tx
                        .send(ChannelMessage::UserUndeafened(
                            new_user.name.clone(),
                            new_voice_channel.clone().unwrap().name.clone(),
                        ))
                        .await
                        .unwrap();
                    self.ctx.request_repaint();
                }
            } else if old_state.unwrap().channel_id != new_state.channel_id {
                push_notification(&format!(
                    "{} moved from {} to {}",
                    new_user.name.clone(),
                    old_voice_channel.clone().unwrap().name.clone(),
                    new_voice_channel.clone().unwrap().name.clone()
                ));
                play_sound();

                self.tx
                    .send(ChannelMessage::UserMoved(
                        new_user.name.clone(),
                        old_voice_channel.clone().unwrap().name.clone(),
                        new_voice_channel.clone().unwrap().name.clone(),
                    ))
                    .await
                    .unwrap();
                self.ctx.request_repaint();
            }
        } else if old_state.is_some() && new_state.channel_id.is_none() {
            push_notification(&format!(
                "{} left {}",
                old_user.clone().unwrap().name.clone(),
                old_voice_channel.clone().unwrap().name.clone()
            ));
            play_sound();

            self.tx
                .send(ChannelMessage::UserLeftChannel(
                    old_user.clone().unwrap().name.clone(),
                    old_voice_channel.clone().unwrap().name.clone(),
                ))
                .await
                .unwrap();
            self.ctx.request_repaint();
        } else {
            println!("Unknown event");
        }
    }
}
