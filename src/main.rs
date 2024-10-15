mod colors;
mod commands;
mod emojis;

use lavalink_rs::{client::LavalinkClient, model::events, node::NodeBuilder, prelude::NodeDistributionStrategy};
use poise::CreateReply;
use poise::serenity_prelude as serenity;
use serenity::{
    all::{CreateEmbed, GatewayIntents},
    Client,
};
use songbird::SerenityInit;
use std::{collections::HashMap, env, sync::Arc, time::Duration, vec};

type Error = Box<dyn std::error::Error + Send + Sync>;
#[allow(dead_code)] // Context isn't being used by this file but is being used by the commands module :)
type Context<'a> = poise::Context<'a, Data, Error>;

// User data that is carried with each invocation
pub struct Data {
    pub lavalink: LavalinkClient
}

// Error handler
async fn on_error(error: poise::FrameworkError<'_, Data, Error>) {
    match error {
        poise::FrameworkError::Setup { error, .. } => panic!("Failed to start bot: {:?}", error),
        poise::FrameworkError::Command { error, ctx, .. } => {
            eprintln!("Error in command `{}`: {:?}", ctx.command().name, error,);
        }
        error => {
            if let Err(e) = poise::builtins::on_error(error).await {
                eprintln!("Error while handling error: {}", e)
            }
        }
    }
}

/// Get help on a command
#[poise::command(prefix_command, slash_command, category = "Utility")]
pub async fn help(
    ctx: Context<'_>,
    #[description = "Command to show help about"]
    #[autocomplete = "poise::builtins::autocomplete_command"]
    command: Option<String>,
) -> Result<(), Error> {
    let listedcommands = &ctx.framework().options.commands;
    match command {
        Some(query) => {
            // Single command
            let foundcommand = poise::find_command(listedcommands, &query, true, &mut vec![])
                .expect("Couldn't find any command");
            let embed = CreateEmbed::new()
                .title(format!(
                    "{} {}",
                    emojis::GEAR,
                    foundcommand.0.qualified_name
                ))
                .color(colors::COLOR0)
                .description(format!(
                    ">>> {}",
                    match foundcommand.0.description.clone() {
                        Some(d) => d,
                        None => "No description".to_string(),
                    }
                ));
            ctx.send(CreateReply {
                embeds: vec![embed],
                ..Default::default()
            })
            .await?;
        }
        None => {
            // All commands
            let mut categories: HashMap<String, Vec<_>> = HashMap::new();
            for cmd in listedcommands {
                let cat = cmd
                    .category
                    .to_owned()
                    .unwrap_or_else(|| "Uncategorized".to_string());
                match categories.get_mut(cat.as_str()) {
                    Some(cmds) => {
                        cmds.push(cmd);
                    }
                    None => {
                        categories.insert(cat.to_string(), vec![cmd]);
                    }
                }
            }
            let mut fields: Vec<(String, String, bool)> = Vec::new();

            for (cat, cmds) in categories {
                let mut content = String::from("```\n");
                for cmd in cmds {
                    content.push_str(cmd.qualified_name.as_str());
                    content.push('\n');
                }
                content.push_str("```");
                fields.push((cat, content, true));
            }
            let embed = CreateEmbed::new()
                .title(format!("{} Help", emojis::GEAR))
                .fields(fields)
                .color(colors::COLOR0);

            ctx.send(CreateReply {
                embeds: vec![embed],
                ..Default::default()
            })
            .await?;
        }
    }
    Ok(())
}

// Main loop
#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Error> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        panic!("Missing argument: TOKEN");
    }
    let token: String = args[1].clone();
    let intents: GatewayIntents = GatewayIntents::default() | GatewayIntents::MESSAGE_CONTENT;

    let framework: poise::Framework<Data, Error> = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            prefix_options: poise::PrefixFrameworkOptions {
                prefix: Some(String::from("r?")),
                additional_prefixes: vec![poise::Prefix::Literal("R?")],
                edit_tracker: Some(Arc::new(poise::EditTracker::for_timespan(
                    Duration::from_secs(3600),
                ))),
                ..Default::default()
            },
            // List commands here
            commands: vec![
                help(),
                commands::utility::ping(),
                commands::music::play(),
                commands::music::join(),
                commands::music::leave()
            ],
            on_error: |error: poise::FrameworkError<Data, Error>| Box::pin(on_error(error)),
            ..Default::default()
        })
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;

                // Call events when a song starts playing, ect..
                let events = events::Events {
                    raw: None,
                    track_start: None,
                    ready: None,
                    ..Default::default()
                };

                // Server node to connect to
                let node_local = NodeBuilder {
                    hostname: "localhost:2333".to_string(),
                    is_ssl: false,
                    events: events::Events::default(),
                    password: env::var("LAVALINK_PASSWORD").expect("Missing lavalink password !"),
                    user_id: ctx.cache.current_user().id.get().into(),
                    session_id: None
                };

                // Lavalink client
                let client = LavalinkClient::new(events, vec![node_local], NodeDistributionStrategy::round_robin()).await;

                Ok(Data {lavalink: client})
            })
        })
        .build();

    let mut client: Client = Client::builder(token, intents)
        .register_songbird()
        .framework(framework)
        .await?;

    client.start().await?;

    Ok(())
}
