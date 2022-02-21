use std::time::Duration;
use serenity::{
    Result as SerenityResult,
    model::{channel::Message, gateway::Ready, id::GuildId},
    client::{Client, Context, EventHandler},
        framework::{
            standard::{
                macros::{command, group, hook},
                Args, CommandResult,
            },
        },
};



#[group]
#[only_in(guilds)]
#[commands(join, leave, play, now_playing, skip, seek, qu)]
struct Music;

#[command]
async fn join(ctx: &Context, msg: &Message) -> CommandResult {
    let guild = msg.guild(&ctx.cache).await.unwrap();
    let guild_id = guild.id;

    let channel_id = guild
        .voice_states
        .get(&msg.author.id)
        .and_then(|voice| voice.channel_id);

    let connect_to = match channel_id {
        Some(channel) => channel,
        None => {
            check_msg(msg.reply(&ctx.http, "join a channel first").await);
            return Ok(());
        }
    };

    let manager = songbird::get(ctx).await.unwrap().clone();
    let (_, handler) = manager.join_gateway(guild_id, connect_to).await;
    match handler {
        Ok(c_info) => {
            let data = ctx.data.read().await;
            let lava_client = data.get::<crate::handlers::Lavalink>().unwrap().clone();
            lava_client.create_session_with_songbird(&c_info).await?;
            info!("Joined {}", msg.channel_id);
        }

        Err(e) => check_msg(
            msg.channel_id
            .say(&ctx.http, format!("cant join channel: {}", e))
            .await,
            )
    }
    Ok(())
}

#[command]
#[aliases(fuckoff)]
async fn leave(ctx: &Context, msg: &Message) -> CommandResult {
    let guild = msg.guild(&ctx.cache).await.unwrap();
    let guild_id = guild.id;

    let manager = songbird::get(ctx).await.unwrap().clone();
    let has_handler = manager.get(guild_id).is_some();

    if has_handler {
        if let Err(e) = manager.remove(guild_id).await {
            check_msg(
                msg.channel_id
                    .say(&ctx.http, format!("Failed: {:?}", e))
                    .await,
            );
        }

        {
            let data = ctx.data.read().await;
            let lava_client = data.get::<crate::handlers::Lavalink>().unwrap().clone();
            lava_client.destroy(guild_id).await?;
        }

        info!("Left {}", msg.channel_id);
    } else {
        check_msg(msg.reply(&ctx.http, "kill yourself NOW!").await);
    }

    Ok(())
}
// TODO make sure bot verifies the user in the bots channel
#[command]
#[min_args(1)]
async fn play(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let query = args.message().to_string();
    let lava_client = {
        let data = ctx.data.read().await;
        data.get::<crate::handlers::Lavalink>().unwrap().clone()
    }; 
    let guild_id = match ctx.cache.guild_channel(msg.channel_id).await {
        Some(channel) => channel.guild_id,
        None => {
            check_msg(
                msg.channel_id
                .say(&ctx.http, "error finding channel info")
                .await,
            );
            return Ok(());
        }
    };
    let manager = songbird::get(ctx).await.unwrap().clone();
    if let None = manager.get(guild_id) {
        let guild = msg.guild(&ctx.cache).await.unwrap();
        let guild_id = guild.id;

            let channel_id = guild
            .voice_states
            .get(&msg.author.id)
            .and_then(|voice| voice.channel_id);

        let connect_to = match channel_id {
            Some(channel) => channel,
            None => {
                check_msg(msg.reply(&ctx.http, "join a channel first").await);
                return Ok(());
            }
        };

        let manager = songbird::get(ctx).await.unwrap().clone();
        let (_, handler) = manager.join_gateway(guild_id, connect_to).await;
        match handler {
            Ok(c_info) => {
                let data = ctx.data.read().await;
                let lava_client = data.get::<crate::handlers::Lavalink>().unwrap().clone();
                lava_client.create_session_with_songbird(&c_info).await?;
                info!("Joined {}", msg.channel_id);
            }

            Err(e) => check_msg(
                msg.channel_id
                .say(&ctx.http, format!("cant join channel: {}", e))
                .await,
                )
        }
    }
    if let Some(_handler) = manager.get(guild_id) {
        let query_info = lava_client.auto_search_tracks(&query).await?;
        if query_info.tracks.is_empty() {
            check_msg(
                msg.channel_id
                .say(&ctx, "could not find any video of the search query")
                .await,
            );
            return Ok(())
        }
        if let Err(e) = &lava_client
            .play(guild_id, query_info.tracks[0].clone())
            .queue()
            .await {
            error!("{}", e);
            return Ok(());
        };
        check_msg(
            msg.channel_id
            .say(
                &ctx.http,
                format!("added: `{}`",query_info.tracks[0].info.as_ref().unwrap().title),
            ).await
        );
    } else {
        check_msg(
            msg.channel_id
            .say(
                &ctx.http,
                "Cant play for some reason!"
            ).await
        );
    }
    Ok(())
}

#[command]
async fn skip(ctx: &Context, msg: &Message) -> CommandResult {
    let data = ctx.data.read().await;
    let lava_client = data.get::<crate::handlers::Lavalink>().unwrap().clone();
    if let Some(track) = lava_client.skip(msg.guild_id.unwrap()).await {
        check_msg(
            msg.channel_id
            .say(
                ctx,
                format!("skipped: `{}`", track.track.info.as_ref().unwrap().title)
            ).await,
        );
        
        if let Some(node) = lava_client.nodes().await.get(&msg.guild_id.unwrap().0) {
            if node.queue.len() == 0 {
                lava_client.stop(msg.guild(&ctx.cache).await.unwrap().id).await.unwrap(); 
            }
        }
    } else {
        check_msg(msg.channel_id.say(&ctx.http, "nothing to skip").await);
    }

    Ok(())
}

#[command]
#[aliases(np, nowplaying)]
async fn now_playing(ctx: &Context, msg: &Message) -> CommandResult {
    let data = ctx.data.read().await;
    let lava_client = data.get::<crate::handlers::Lavalink>().unwrap().clone();

    if let Some(node) = lava_client.nodes().await.get(&msg.guild_id.unwrap().0) {
        if let Some(track) = &node.now_playing {
            check_msg(
                msg.channel_id
                    .say(
                        &ctx,
                        format!("now playing: `{}`\n**{}** / **{}**", 
                            track.track.info.as_ref().unwrap().title,
                            ms_to_hms(track.track.info.as_ref().unwrap().position),
                            ms_to_hms(track.track.info.as_ref().unwrap().length
                            )
                    )).await);
        } else { 
            check_msg(
                msg.channel_id.say(&ctx.http, "Kill yourself NOW!").await,);
        }
    } else {
        check_msg(
            msg.channel_id.say(&ctx.http, "Kill yourself NOW!").await,
        );
    }
    Ok(())
}
#[command]
#[min_args(1)]
async fn seek(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let duration = hms_to_duration(args.message()).unwrap();
    let data = ctx.data.read().await;
    let lava_client = data.get::<crate::handlers::Lavalink>().unwrap().clone();

    if let Ok(_) = lava_client.scrub(msg.guild_id.unwrap(), duration).await {
        info!("Scrubbing to {:?}", duration);
        check_msg(
            msg.channel_id
            .say(
                &ctx.http,
                format!("scrubbin")
            ).await
        );
    } else {
        check_msg(
            msg.channel_id
            .say(
                &ctx.http,
                "do you feel safe in your house"
            ).await
        );
       
    }
   Ok(()) 
}
#[command]
#[aliases(queue)]
async fn qu(ctx: &Context, msg: &Message) -> CommandResult {
    let data = ctx.data.read().await;
    let lava_client = data.get::<crate::handlers::Lavalink>().unwrap().clone();
    if let Some(node) = lava_client.nodes().await.get(&msg.guild_id.unwrap().0) {
        for track in &node.queue {
            check_msg(
                msg.channel_id
                .say(
                    &ctx.http,
                    format!("`{}`", track.track.info.as_ref().unwrap().title)

                ).await
            );
        }
        return Ok(())
    } else {
        check_msg(
            msg.channel_id
            .say(
                &ctx.http,
                "you are insufferable"
            ).await
        );
    }
    Ok(())
}

fn check_msg(result: SerenityResult<Message>) {
    if let Err(why) = result {
        error!("Error sending message: {:?}", why);
    }
}

fn ms_to_hms(ms: u64) -> String {
    let duration = Duration::new(ms/1000, 0);
    let seconds = duration.as_secs() % 60;
    let minutes = (duration.as_secs() / 60) % 60;
    let hours = (duration.as_secs() / 60) / 60;
    let (mut hours_str, mut minutes_str, mut seconds_str) = (hours.to_string(), minutes.to_string(),seconds.to_string()); 
    if seconds < 10 {seconds_str=format!("0{}", seconds)}
    if minutes < 10 {minutes_str=format!("0{}", minutes)}
    if hours < 10 {hours_str=format!("0{}", hours)}
    format!("{}:{}:{}", hours_str, minutes_str, seconds_str)
}

fn hms_to_duration(hms: &str) -> Result<Duration, ()> {
    let v: Vec<u64> = hms
        .split(':')
        .map(|x| x.parse::<u64>().unwrap())
        .collect();

    match v.len() {
        // seconds
        1 => {
            Ok(Duration::new(v[0], 0))
        }
        // minutes and seconds
        2 => {
            let seconds = (v[0] * 60) + (v[1]);
            Ok(Duration::new(seconds, 0))
        }
        // hours minutes and seconds
        3 => {
            let seconds = (v[0] * 3600) + (v[1] * 60) + v[0];
            Ok(Duration::new(seconds, 0))
        }
        _ => Err(())
        
    }
}
