use std::{time::Duration, ptr::read};
use serenity::{
    prelude::*,
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
//TODO dont join everytime a song is played. if in channel just play the song.
#[command]
#[min_args(1)]
async fn play(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let query = args.message().to_string();
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
    let mut lava_client = {
        let data = ctx.data.read().await;
        data.get::<crate::handlers::Lavalink>().unwrap().clone()
    };
    let manager = songbird::get(ctx).await.unwrap().clone();
    let (_, handler) = manager.join_gateway(guild_id, connect_to).await;
    match handler {
        Ok(c_info) => {
            let data = ctx.data.read().await;
            lava_client = data.get::<crate::handlers::Lavalink>().unwrap().clone();
            lava_client.create_session_with_songbird(&c_info).await?;
            info!("Joined to play in {}", msg.channel_id);
        }

        Err(e) => check_msg(
            msg.channel_id
            .say(&ctx.http, format!("cant join channel: {}", e))
            .await,
            )
    }
    if let Some(_handler) = manager.get(guild_id) {
            let query_info = lava_client.auto_search_tracks(&query).await?;
            if query_info.tracks.is_empty() {
                check_msg(
                    msg.channel_id
                    .say(&ctx.http, "could not find any video of the search query")
                    .await,
                );
                return Ok(());
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
                    format!(
                        "queued: `{}`",
                        query_info.tracks[0].info.as_ref().unwrap().title
                    ),
                ).await
            );

            
    };


    Ok(())
}

//TODO make this stop when last song is skipped
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
            dbg!(check_msg(
                msg.channel_id
                    .say(
                        &ctx,
                        format!("now playing: `{}`\n**{}** / **{}**", 
                            track.track.info.as_ref().unwrap().title,
                            ms_to_hms(track.track.info.as_ref().unwrap().position),
                            ms_to_hms(track.track.info.as_ref().unwrap().length
                            )
                    )).await));
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
    let duration = Duration::new(args.message().parse::<u64>().unwrap(), 0);
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

#[cfg(test)]
mod tests {
    use crate::music::ms_to_hms;

    #[test]
    fn returns_correct_string() {
        let ms = 4000000;
        assert_eq!(ms_to_hms(ms), format!("01:06:40"));
    }

    #[test]
    fn another_time() {
        let ms = 9034421343;
        assert_eq!(ms_to_hms(ms), format!("2509:33:41"));
    }
}
