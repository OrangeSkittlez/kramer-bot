use serenity::prelude::*;
use serenity::{
    async_trait,
    client::{Context, EventHandler},
    model::{gateway::Ready, id::GuildId},
};
use lavalink_rs::{gateway::*, model::*, LavalinkClient};

pub struct Lavalink;
impl TypeMapKey for Lavalink {
    type Value = LavalinkClient;
}
pub struct Handler;
pub struct LavalinkHandler;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, _: Context, ready: Ready) {
        info!("{} is connected!", ready.user.name);
    }

    async fn cache_ready(&self, _: Context, _guilds: Vec<GuildId>) {
    info!("Cache is ready!");
    }
}

#[async_trait]
impl LavalinkEventHandler for LavalinkHandler {
    async fn track_start(&self, _client: LavalinkClient, event: TrackStart) {
        info!("Track started!\nGuild: {}", event.guild_id);
    }
    
    async fn track_finish(&self, _client: LavalinkClient, event: TrackFinish) {
        info!("Track finish!\nGuild: {}", event.guild_id);
    }
}

