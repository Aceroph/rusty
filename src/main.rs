mod commands;

use serenity::{all::GatewayIntents, Client};
use std::{env, sync::Arc, time::Duration};
use tokio;

type Error = Box<dyn std::error::Error + Send + Sync>;
#[allow(dead_code)] // Context isn't being used by this file but is being used by the commands module :)
type Context<'a> = poise::Context<'a, (), Error>;

async fn on_error(error: poise::FrameworkError<'_, (), Error>) {
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

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        panic!("Missing argument: TOKEN");
    }
    let token: String = args[1].clone();
    let intents: GatewayIntents = GatewayIntents::default();

    let framework: poise::Framework<(), Error> = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            prefix_options: poise::PrefixFrameworkOptions {
                prefix: Some("r?".into()),
                edit_tracker: Some(Arc::new(poise::EditTracker::for_timespan(
                    Duration::from_secs(3600),
                ))),
                ..Default::default()
            },
            commands: vec![commands::help()],
            on_error: |error| Box::pin(on_error(error)),
            ..Default::default()
        })
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                Ok(())
            })
        })
        .build();

    let mut client: Client = Client::builder(token, intents)
        .framework(framework)
        .await
        .expect("Failed to create client");

    if let Err(why) = client.start().await {
        eprintln!("Failed to start client: {:?}", why)
    }
}