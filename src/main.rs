mod handlers;
mod music;
mod general;
#[macro_use]
extern crate tracing;
use std::env;
use serenity::{
    client::{Client, Context, EventHandler},
    framework::{StandardFramework, standard::{macros::hook, CommandResult}},
    http::Http,
    model::{channel::Message, gateway::Ready, id::GuildId, misc::Mentionable},
    Result as SerenityResult,
};
use songbird::SerenityInit;

use crate::music::MUSIC_GROUP;
use crate::general::GENERAL_GROUP;


#[hook]
async fn after(_ctx: &Context, _msg: &Message, command_name: &str, command_result: CommandResult) {
    match command_result {
        Err(e) => info!(
            "Command '{}' has returned error {:?} => {}",
            command_name, e, e
        ),
        _ => (),
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env::set_var("RUST_LOG", "info,lavalink_rs=debug"); dotenv::dotenv().ok();
    tracing_subscriber::fmt::init();
    info!("Tracing initialized!");
    
    let token = env::var("DISCORD_TOKEN").expect("Put your discord token in the env file!");
    let http = Http::new_with_token(&token);
    let bot_id = match http.get_current_application_info().await {
        Ok(info) => info.id,
        Err(e) => panic!("Could not access app info! {:?}", e),
    };

    let framework = StandardFramework::new()
        .configure(|c| c.prefix(env::var("PREFIX").unwrap()))
        .after(after)
        .group(&MUSIC_GROUP)
        .group(&GENERAL_GROUP);
    
    let mut client = Client::builder(&token)
        .event_handler(handlers::Handler)
        .framework(framework)
        .register_songbird()
        .await
        .expect("Error creating client!");

    let lava_client = lavalink_rs::LavalinkClient::builder(bot_id)
        .set_host("127.0.0.1")
            .set_password(env::var("LAVALINK_PASSWORD").unwrap())
            .build(handlers::LavalinkHandler)
            .await?;
    {
        let mut data = client.data.write().await;
        data.insert::<handlers::Lavalink>(lava_client);
    }

    let _ = client
        .start()
        .await
        .map_err(|e| warn!("Client ended {:?}", e));
    Ok(())
}
