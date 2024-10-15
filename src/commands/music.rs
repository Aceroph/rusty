use std::ops::Deref;

use crate::{Context, Error};
use poise::serenity_prelude as serenity;
use serenity::all::{ChannelId, Http, Mentionable};

// Makes the bot join the specified channel
async fn _join(
    ctx: &Context<'_>,
    guild_id: serenity::GuildId,
    channel_id: Option<serenity::ChannelId>
) -> Result<bool, Error> {
    let lava_client = ctx.data().lavalink.clone();
    let manager = songbird::get(ctx.serenity_context()).await.unwrap().clone();

    if lava_client.get_player_context(Into::<u64>::into(guild_id)).is_none() {
        let connect_to = match channel_id {
            Some(x) => x,
            None => {
                let guild = ctx.guild().unwrap().deref().clone();
                let user_channel_id = guild
                    .voice_states
                    .get(&ctx.author().id)
                    .and_then(|voice_state| voice_state.channel_id);

                match user_channel_id {
                    Some(channel) => channel,
                    None => {
                        ctx.say("Not in a voice channel !").await?;

                        return Err("Not in a voice channel !".into());
                    }
                }
            }
        };

        let handler = manager.join_gateway(guild_id, channel_id.unwrap()).await;
        match handler {
            Ok((connection_info, _)) => {
                lava_client.create_player_context_with_data::<(ChannelId, std::sync::Arc<Http>)>(guild_id, connection_info, std::sync::Arc::new((
                    ctx.channel_id(),
                    ctx.serenity_context().http.clone(),
                )),).await?;

                ctx.say(format!("Joined {}", connect_to.mention())).await?;
                return Ok(true);
            }
            Err(why) => {
                ctx.say(format!("Error while joining channel : {}", why)).await?;
                return Err(why.into());
            }
        }
    }
    Ok(false)
}

/// Plays music in the channel
#[poise::command(prefix_command, slash_command, category = "Music")]
pub async fn play(
    ctx: Context<'_>,
    #[description = "Song searching query"]
    #[rest]
    query: Option<String>
) -> Result<(), Error> {
    match query {
        Some(query) => {
            ctx.send(poise::CreateReply {
                content: Some(format!(
                    "Searching for songs matching: `{query:?}`\n-# Not actually searching rn"
                )),
                ..Default::default()
            })
            .await?;
        }
        None => {
            ctx.send(poise::CreateReply {
                content: Some("No query provided, unpausing..".into()),
                ..Default::default()
            })
            .await?;
        }
    }
    Ok(())
}

/// Joins the channel
#[poise::command(prefix_command, slash_command, category = "Music")]
pub async fn join(
    ctx: Context<'_>,
    #[description = "The channel to join"]
    #[channel_types("Voice")]
    channel: Option<ChannelId>
) -> Result<(), Error> {
    let guild_id = ctx.guild_id().unwrap();

    _join(&ctx, guild_id, channel).await?;

    Ok(())
}

/// Leaves the current channel
#[poise::command(prefix_command, slash_command, category = "Music")]
pub async fn leave(
    ctx: Context<'_>
) -> Result<(), Error> {
    let guild_id = ctx.guild_id().unwrap();

    let manager = songbird::get(ctx.serenity_context()).await.unwrap().clone();
    let lava_client = ctx.data().lavalink.clone();

    lava_client.delete_player(guild_id).await?;

    if manager.get(guild_id).is_some() {
        manager.remove(guild_id).await?;
    }

    ctx.say("Left voice channel.").await?;

    Ok(())
}
