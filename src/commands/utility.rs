use poise::serenity_prelude as serenity;
use serenity::all::{CreateEmbed, Timestamp};

use crate::{colors, emojis, Context, Error};

// Notice how the comment below has an extra slash? That's a doc comment and it'll also serve as the command description
/// Send the bot latency in ms
#[poise::command(prefix_command, slash_command, category = "Utility")]
pub async fn ping(ctx: Context<'_>) -> Result<(), Error> {
    let latency = ctx.ping().await.as_millis();
    let embed = CreateEmbed::new()
        .title(format!("{} Pong !", emojis::PINGPONG))
        .description(format!("{}latency: `{}` ms", emojis::SPACE, latency))
        .timestamp(Timestamp::now())
        .color(colors::COLOR0);

    ctx.send(poise::CreateReply {
        embeds: vec![embed],
        reply: true,
        ..Default::default()
    })
    .await?;
    Ok(())
}
