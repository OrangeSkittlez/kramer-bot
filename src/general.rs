use serenity::{
model::{channel::Message, gateway::Ready, id::GuildId},
client::Context,
    framework::{
        standard::{
            macros::{command, group,},
            Args, CommandResult,
        },
    },
};

#[group]
#[commands(set_prefix)]
struct General;

#[command]
async fn set_prefix(ctx: &Context, msg: &Message,mut args: Args) -> CommandResult {
    let prefix:String = args.single().unwrap();
    std::env::set_var("PREFIX", &prefix);
    msg.channel_id.say(&ctx.http, format!("changing prefix to {}", &prefix)).await?;
    Ok(())
}



