mod handler;
mod quit;

use quit::*;

use serenity::{
    client::bridge::gateway::ShardManager, framework::standard::StandardFramework, http::Http,
    prelude::*,
};
use serenity_slash_command_framework::SlashCommandHandler;
use std::{collections::HashSet, sync::Arc};
use tracing::error;
use tracing_subscriber::{EnvFilter, FmtSubscriber};

pub struct ShardManagerContainer;

impl TypeMapKey for ShardManagerContainer {
    type Value = Arc<Mutex<ShardManager>>;
}

#[tokio::main]
async fn main() {
    dotenv::dotenv().expect("Failed to load .env file");

    let subscriber = FmtSubscriber::builder()
        .with_env_filter(EnvFilter::from_default_env())
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("Failed to start the logger");

    let token = std::env::var("DISCORD_TOKEN").expect(
        "Expected a bot token in the environment. Add the `DISCORD_TOKEN` key to the .env file.",
    );

    let http = Http::new_with_token(&token);

    let owners = match http.get_current_application_info().await {
        Ok(info) => {
            let mut owners = HashSet::new();
            owners.insert(info.owner.id);
            owners
        }
        Err(why) => panic!("Could not access application info: {:?}", why),
    };

    let prefix = std::env::var("BOT_PREFIX").expect(
        "Expected a command prefix in the environment. Add the `BOT_PREFIX` key to the .env file.",
    );

    let framework = StandardFramework::new().configure(|c| c.owners(owners).prefix(&prefix));

    let mut handler = handler::Handler::default();
    handler
        .register_slash_command::<QUIT_COMMAND>()
        .await;

    let application_id: u64 = std::env::var("APPLICATION_ID")
        .expect("Expected an application id in the environment. Add the `APPLICATION_ID` key to the .env file")
        .parse()
        .expect("Invalid application id");

    let mut client = Client::builder(token)
        .event_handler(handler)
        .framework(framework)
        .application_id(application_id)
        .await
        .expect("Error creating client");

    {
        let mut data = client.data.write().await;
        data.insert::<ShardManagerContainer>(client.shard_manager.clone());
    }

    if let Err(why) = client.start().await {
        error!("Client error: {:?}", why);
    }
}
