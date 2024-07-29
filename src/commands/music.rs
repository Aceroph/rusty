use crate::{Context, Error};

#[poise::command(prefix_command, slash_command, category = "Music")]
pub async fn play(
    ctx: Context<'_>,
    #[description = "Song searching query"]
    #[rest]
    query: Option<String>,
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
                content: Some(format!("No query provided, unpausing..")),
                ..Default::default()
            })
            .await?;
        }
    }
    Ok(())
}
